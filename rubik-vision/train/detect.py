import argparse
from pathlib import Path

import cv2
import numpy as np
from ultralytics import YOLO

DEFAULT_MODEL = Path(__file__).parent.parent.parent / "runs" / "pose" / "rubik" / "rubik" / "weights" / "best.pt"
WIN_NAME = "Rubik Detection"
KP_COLORS = [
    (0, 0, 255),
    (0, 255, 0),
    (255, 0, 0),
    (0, 165, 255),
]

MIN_SQUARENESS = 0.4   # min(w,h)/max(w,h) of bounding box
SIZE_RATIO_MIN = 0.05   # blob area / face area
SIZE_RATIO_MAX = 0.15

RUBIK_COLORS = {
    "W": (255, 255, 255),
    "Y": (0,   255, 255),
    "R": (0,   0,   200),
    "O": (0,   128, 255),
    "B": (150, 0,   0  ),
    "G": (0,   180, 0  ),
}

def _to_lab(bgr):
    return cv2.cvtColor(np.array([[bgr]], dtype=np.uint8), cv2.COLOR_BGR2LAB)[0, 0].astype(np.float32)

RUBIK_COLORS_LAB = {k: _to_lab(v) for k, v in RUBIK_COLORS.items()}


def sample_color(frame, center, radius):
    cx, cy = int(round(center[0])), int(round(center[1]))
    r = max(1, int(radius))
    h, w = frame.shape[:2]
    region = frame[max(0, cy - r):min(h, cy + r), max(0, cx - r):min(w, cx + r)]
    if region.size == 0:
        return None
    return tuple(int(c) for c in region.mean(axis=(0, 1)))


def classify_all(labs):
    """Classify all cells with per-frame L normalization (white-point correction)."""
    valid = np.array([l for l in labs if l is not None], dtype=np.float32)
    if len(valid) == 0:
        return [("?", (80, 80, 80))] * len(labs)

    max_L = valid[:, 0].max()
    L_scale = 255.0 / max_L if max_L > 1 else 1.0

    results = []
    for lab in labs:
        if lab is None:
            results.append(("?", (80, 80, 80)))
            continue
        corrected = np.array([np.clip(lab[0] * L_scale, 0, 255), lab[1], lab[2]], dtype=np.float32)
        label = min(RUBIK_COLORS_LAB, key=lambda k: float(np.linalg.norm(corrected - RUBIK_COLORS_LAB[k])))
        results.append((label, RUBIK_COLORS[label]))

    return results


def compute_grid_centers(corners):
    """
    Map 3x3 cell centers from normalized grid coords to image space.
    corners: BL(0), BR(1), TL(2), TR(3) as (x, y).
    Returns 9 (x, y) tuples, row-major from top-left.
    """
    src = np.array([[0, 0], [1, 0], [0, 1], [1, 1]], dtype=np.float32)
    # (0,0)→TL, (1,0)→TR, (0,1)→BL, (1,1)→BR
    dst = np.array([corners[2], corners[3], corners[0], corners[1]], dtype=np.float32)
    H, _ = cv2.findHomography(src, dst)
    centers = []
    for row in range(3):
        for col in range(3):
            pt = np.array([[[(col + 0.5) / 3, (row + 0.5) / 3]]], dtype=np.float32)
            mapped = cv2.perspectiveTransform(pt, H)
            centers.append((float(mapped[0, 0, 0]), float(mapped[0, 0, 1])))
    return centers


def quad_area(corners):
    """Shoelace area of quadrilateral BL, BR, TR, TL."""
    pts = [corners[0], corners[1], corners[3], corners[2]]
    n = len(pts)
    a = sum(pts[i][0] * pts[(i + 1) % n][1] - pts[(i + 1) % n][0] * pts[i][1]
            for i in range(n))
    return abs(a) / 2


def detect_blobs(frame, centers, face_area):
    """
    Flood-fill from each cell center in LAB space.
    Returns a list of dicts (or None) per center.
    """
    lab = cv2.cvtColor(frame, cv2.COLOR_BGR2LAB)

    h, w = frame.shape[:2]
    # FIXED_RANGE: compare each pixel against the seed (not its neighbour)
    flags = 4 | (255 << 8) | cv2.FLOODFILL_MASK_ONLY | cv2.FLOODFILL_FIXED_RANGE

    blobs = []
    for cx, cy in centers:
        seed = (int(round(cx)), int(round(cy)))
        if not (0 <= seed[0] < w and 0 <= seed[1] < h):
            blobs.append(None)
            continue

        mask = np.zeros((h + 2, w + 2), np.uint8)
        cv2.floodFill(lab, mask, seed, (255, 255, 255),
                      loDiff=(50, 25, 25),   # L loose (handles reflections), A/B tight (stops at color boundaries)
                      upDiff=(50, 25, 25),
                      flags=flags)
        blob_mask = (mask[1:-1, 1:-1] == 255).astype(np.uint8) * 255

        contours, _ = cv2.findContours(blob_mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        if not contours:
            blobs.append(None)
            continue

        cnt = max(contours, key=cv2.contourArea)
        area = cv2.contourArea(cnt)
        _, _, bw, bh = cv2.boundingRect(cnt)
        squareness = min(bw, bh) / max(bw, bh) if max(bw, bh) > 0 else 0
        size_ratio = area / face_area if face_area > 0 else 0

        if squareness < MIN_SQUARENESS or not (SIZE_RATIO_MIN <= size_ratio <= SIZE_RATIO_MAX):
            blobs.append(None)
            continue

        pixels = frame[blob_mask > 0]
        color = tuple(int(c) for c in pixels.mean(axis=0)) if len(pixels) > 0 else (128, 128, 128)
        M = cv2.moments(cnt)
        centroid = (M["m10"] / M["m00"], M["m01"] / M["m00"]) if M["m00"] > 0 else (cx, cy)
        blobs.append({"contour": cnt, "color": color, "centroid": centroid})

    return blobs


def refine_grid_from_blobs(blobs, initial_centers):
    """
    Least-squares homography fit from normalized grid coords to image space,
    using detected blob centroids as ground truth. Falls back to initial_centers
    if fewer than 4 blobs were detected.
    """
    src, dst = [], []
    for i, blob in enumerate(blobs):
        if blob is None:
            continue
        row, col = divmod(i, 3)
        src.append([(col + 0.5) / 3, (row + 0.5) / 3])
        dst.append(blob["centroid"])

    if len(src) < 4:
        return initial_centers

    H, _ = cv2.findHomography(
        np.array(src, dtype=np.float32),
        np.array(dst, dtype=np.float32),
        method=0,  # plain least squares over all inliers
    )
    if H is None:
        return initial_centers

    grid_pts = np.array(
        [[[(col + 0.5) / 3, (row + 0.5) / 3]] for row in range(3) for col in range(3)],
        dtype=np.float32,
    )
    refined = cv2.perspectiveTransform(grid_pts, H)
    return [(float(refined[i, 0, 0]), float(refined[i, 0, 1])) for i in range(9)]


def draw(frame, centers, blobs, cell_radius):
    vis = frame.copy()
    r = max(4, int(cell_radius * 0.55))
    font = cv2.FONT_HERSHEY_SIMPLEX
    font_scale = max(0.3, cell_radius * 0.03)

    samples = [sample_color(frame, c, cell_radius * 0.7) for c in centers]
    labs = [_to_lab(s) if s is not None else None for s in samples]
    labels = classify_all(labs)

    for (cx, cy), _, (label, ref_bgr) in zip(centers, blobs, labels):
        cx, cy = int(cx), int(cy)

        cv2.rectangle(vis, (cx - r, cy - r), (cx + r, cy + r), ref_bgr, -1)
        cv2.rectangle(vis, (cx - r, cy - r), (cx + r, cy + r), (255, 255, 255), 1)
        (tw, th), _ = cv2.getTextSize(label, font, font_scale, 1)
        cv2.putText(vis, label, (cx - tw // 2, cy + th // 2), font, font_scale, (0, 0, 0), 2)
        cv2.putText(vis, label, (cx - tw // 2, cy + th // 2), font, font_scale, (255, 255, 255), 1)

    return vis


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", default=str(DEFAULT_MODEL))
    parser.add_argument("--camera", type=int, default=0)
    parser.add_argument("--conf", type=float, default=0.5)
    args = parser.parse_args()

    model = YOLO(args.model)

    cap = cv2.VideoCapture(args.camera)
    if not cap.isOpened():
        print("Cannot open webcam")
        return

    cv2.namedWindow(WIN_NAME, cv2.WINDOW_NORMAL)

    while True:
        ret, frame = cap.read()
        if not ret:
            break

        results = model(frame, conf=args.conf, verbose=False)
        vis = frame.copy()

        for result in results:
            if result.keypoints is None:
                continue
            for kps in result.keypoints.xy:
                if len(kps) != 4:
                    continue
                raw = kps.cpu().numpy().astype(np.float32)
                centers = compute_grid_centers(raw)
                fa = quad_area(raw)
                cell_radius = (fa / 9) ** 0.5 / 2
                blobs = detect_blobs(frame, centers, fa)
                centers = refine_grid_from_blobs(blobs, centers)
                vis = draw(vis, centers, blobs, cell_radius)

        cv2.imshow(WIN_NAME, vis)
        if cv2.waitKey(1) & 0xFF == ord("q"):
            break

    cap.release()
    cv2.destroyAllWindows()


if __name__ == "__main__":
    main()
