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


SEARCH_MARGIN = 80  # px around YOLO bbox to search for lines


def line_intersect(p1, p2, p3, p4):
    """Return intersection of lines (p1,p2) and (p3,p4), or None if parallel."""
    x1, y1 = p1; x2, y2 = p2; x3, y3 = p3; x4, y4 = p4
    denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4)
    if abs(denom) < 1e-6:
        return None
    t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom
    return (x1 + t * (x2 - x1), y1 + t * (y2 - y1))


def refine_with_hough(gray, raw_pts):
    xs = raw_pts[:, 0]; ys = raw_pts[:, 1]
    x1 = int(max(0, xs.min() - SEARCH_MARGIN))
    y1 = int(max(0, ys.min() - SEARCH_MARGIN))
    x2 = int(min(gray.shape[1], xs.max() + SEARCH_MARGIN))
    y2 = int(min(gray.shape[0], ys.max() + SEARCH_MARGIN))

    roi = gray[y1:y2, x1:x2]
    edges = cv2.Canny(roi, 50, 150)
    lines_raw = cv2.HoughLinesP(edges, 1, np.pi / 180,
                                 threshold=40, minLineLength=30, maxLineGap=10)
    if lines_raw is None:
        return raw_pts, None

    # shift line endpoints back to full-image coordinates
    lines = [((x1 + ax, y1 + ay), (x1 + bx, y1 + by))
             for (ax, ay, bx, by) in lines_raw[:, 0]]

    # for each raw corner, find the line intersection closest to it
    refined = []
    for corner in raw_pts:
        best_pt = corner
        best_d = float("inf")
        for i, l1 in enumerate(lines):
            for l2 in lines[i + 1:]:
                pt = line_intersect(l1[0], l1[1], l2[0], l2[1])
                if pt is None:
                    continue
                d = np.hypot(pt[0] - corner[0], pt[1] - corner[1])
                if d < best_d and d < SEARCH_MARGIN:
                    best_d = d
                    best_pt = pt
        refined.append(best_pt)

    return np.array(refined, dtype=np.float32), lines


def draw(frame, raw_kps, refined_kps, hough_lines):
    vis = frame.copy()

    if hough_lines is not None:
        for (ax, ay), (bx, by) in hough_lines:
            cv2.line(vis, (ax, ay), (bx, by), (80, 80, 80), 1)

    if refined_kps is not None and len(refined_kps) == 4:
        # keypoint order: BL(0), BR(1), TL(2), TR(3) → wind as BL→BR→TR→TL
        pts = refined_kps[[0, 1, 3, 2]].astype(np.int32).reshape((-1, 1, 2))
        cv2.polylines(vis, [pts], isClosed=True, color=(0, 255, 255), thickness=2)

    for i, (rx, ry) in enumerate(raw_kps):
        cv2.circle(vis, (int(rx), int(ry)), 6, (100, 100, 100), -1)

    if refined_kps is not None:
        for i, (rx, ry) in enumerate(refined_kps):
            cv2.circle(vis, (int(rx), int(ry)), 6, KP_COLORS[i], -1)
            cv2.circle(vis, (int(rx), int(ry)), 7, (255, 255, 255), 1)
            cv2.putText(vis, str(i + 1), (int(rx) + 9, int(ry) + 5),
                        cv2.FONT_HERSHEY_SIMPLEX, 0.5, KP_COLORS[i], 2)

    hint = "q: quit  |  gray dots: YOLO raw  |  color dots: Hough refined  |  gray lines: detected edges"
    cv2.putText(vis, hint, (10, vis.shape[0] - 10),
                cv2.FONT_HERSHEY_SIMPLEX, 0.45, (0, 0, 0), 3)
    cv2.putText(vis, hint, (10, vis.shape[0] - 10),
                cv2.FONT_HERSHEY_SIMPLEX, 0.45, (255, 255, 255), 1)
    return vis


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", default=str(DEFAULT_MODEL))
    parser.add_argument("--camera", type=int, default=0)
    parser.add_argument("--conf", type=float, default=0.25)
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
        gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)

        vis = frame.copy()
        for result in results:
            if result.keypoints is None:
                continue
            for kps in result.keypoints.xy:
                if len(kps) != 4:
                    continue
                raw = kps.cpu().numpy().astype(np.float32)
                refined, hough_lines = refine_with_hough(gray, raw)
                vis = draw(vis, raw, refined, hough_lines)

        cv2.imshow(WIN_NAME, vis)
        if cv2.waitKey(1) & 0xFF == ord("q"):
            break

    cap.release()
    cv2.destroyAllWindows()


if __name__ == "__main__":
    main()
