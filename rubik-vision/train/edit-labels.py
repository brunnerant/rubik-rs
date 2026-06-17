import cv2
from pathlib import Path

ROOT_DIR = Path("../../data/vision")
IMAGES_DIR = ROOT_DIR / "images"
LABELS_DIR = ROOT_DIR / "labels"
IMG_EXTENSIONS = (".jpg", ".jpeg", ".png", ".bmp", ".tiff", ".tif")
WIN_NAME = "Label Editor"
MAX_WIN_W, MAX_WIN_H = 1400, 900

KP_COLORS = [
    (0, 0, 255),    # red
    (0, 255, 0),    # green
    (255, 0, 0),    # blue
    (0, 165, 255),  # orange
]
FONT = cv2.FONT_HERSHEY_SIMPLEX


def get_images():
    paths = []
    for ext in IMG_EXTENSIONS:
        paths.extend(IMAGES_DIR.glob(f"*{ext}"))
        paths.extend(IMAGES_DIR.glob(f"*{ext.upper()}"))
    return sorted(set(paths))


def label_path(img_path: Path):
    return (LABELS_DIR / img_path.stem).with_suffix(".txt")


def load_keypoints(img_path, img_w, img_h):
    path = label_path(img_path)
    if not path.exists():
        return []
    with open(path) as f:
        line = f.readline().strip()
    if not line:
        return []
    parts = line.split()
    kps = []
    if len(parts) == 13:       # no visibility flag
        for i in range(4):
            kps.append((int(float(parts[5 + i * 2]) * img_w),
                        int(float(parts[6 + i * 2]) * img_h)))
    elif len(parts) == 17:     # with visibility flag
        for i in range(4):
            kps.append((int(float(parts[5 + i * 3]) * img_w),
                        int(float(parts[6 + i * 3]) * img_h)))
    return kps


def save_keypoints(img_path, keypoints, img_w, img_h):
    xs = [kp[0] for kp in keypoints]
    ys = [kp[1] for kp in keypoints]
    pad = max(20, img_w * 0.01, img_h * 0.01)
    x1 = max(0.0, min(xs) - pad) / img_w
    y1 = max(0.0, min(ys) - pad) / img_h
    x2 = min(float(img_w), max(xs) + pad) / img_w
    y2 = min(float(img_h), max(ys) + pad) / img_h
    cx, cy = (x1 + x2) / 2, (y1 + y2) / 2
    w, h = x2 - x1, y2 - y1
    kp_str = " ".join(f"{kp[0]/img_w:.6f} {kp[1]/img_h:.6f} 2" for kp in keypoints)
    LABELS_DIR.mkdir(parents=True, exist_ok=True)
    with open(label_path(img_path), "w") as f:
        f.write(f"0 {cx:.6f} {cy:.6f} {w:.6f} {h:.6f} {kp_str}\n")
    print(f"Saved {label_path(img_path)}")


def _s(img_w, img_h):
    """Scale factor: how much to enlarge overlay elements so they appear
    the right size after the window downscales the image."""
    display_scale = min(MAX_WIN_W / img_w, MAX_WIN_H / img_h, 1.0)
    return 1.0 / display_scale


def render(img, keypoints, img_path: Path, idx, total):
    vis = img.copy()
    h, w = vis.shape[:2]
    s = _s(w, h)
    r = max(5, int(s * 3))

    if len(keypoints) == 4:
        xs = [kp[0] for kp in keypoints]
        ys = [kp[1] for kp in keypoints]
        pad = int(max(20, w * 0.01, h * 0.01))
        cv2.rectangle(vis,
                      (max(0, min(xs) - pad), max(0, min(ys) - pad)),
                      (min(w, max(xs) + pad), min(h, max(ys) + pad)),
                      (0, 200, 0), max(1, int(s * 2)))

    for i, kp in enumerate(keypoints):
        cv2.circle(vis, kp, r + int(s * 2), (255, 255, 255), -1)
        cv2.circle(vis, kp, r, KP_COLORS[i], -1)
        cv2.putText(vis, str(i + 1),
                    (kp[0] + int(s * 10), kp[1] + int(s * 6)),
                    FONT, s * 0.6, KP_COLORS[i], max(1, int(s * 2)))

    saved_marker = "  SAVED" if (len(keypoints) == 4 and label_path(img_path).exists()) else ""
    status = f"{idx+1}/{total}  {img_path.name}  [{len(keypoints)}/4 kps]{saved_marker}"
    hint = "click: place kp | r: reset | n/p: next/prev | q: quit"
    for text, y, ts in [(status, int(s * 32), s * 0.8), (hint, h - int(s * 14), s * 0.6)]:
        cv2.putText(vis, text, (int(s * 10), y), FONT, ts, (0, 0, 0), max(2, int(s * 4)))
        cv2.putText(vis, text, (int(s * 10), y), FONT, ts, (255, 255, 255), max(1, int(s * 2)))

    return vis


def main():
    images = get_images()
    if not images:
        print(f"No images found in {IMAGES_DIR}")
        return

    idx = 0
    keypoints = []
    img = None

    def load(i):
        nonlocal img, keypoints
        img = cv2.imread(images[i])
        if img is None:
            print(f"Cannot read {images[i]}")
            return
        ih, iw = img.shape[:2]
        keypoints = load_keypoints(images[i], iw, ih)
        ds = min(MAX_WIN_W / iw, MAX_WIN_H / ih, 1.0)
        cv2.resizeWindow(WIN_NAME, int(iw * ds), int(ih * ds))

    def on_click(event, x, y, flags, param):
        nonlocal keypoints
        if event == cv2.EVENT_LBUTTONDOWN and len(keypoints) < 4:
            keypoints.append((x, y))
            if len(keypoints) == 4:
                ih, iw = img.shape[:2]
                save_keypoints(images[idx], keypoints, iw, ih)

    cv2.namedWindow(WIN_NAME, cv2.WINDOW_NORMAL)
    cv2.setMouseCallback(WIN_NAME, on_click)
    load(idx)

    while True:
        if img is not None:
            cv2.imshow(WIN_NAME, render(img, keypoints, images[idx], idx, len(images)))
        key = cv2.waitKey(20) & 0xFF
        if key == ord("q"):
            break
        elif key == ord("r"):
            keypoints = []
        elif key in (ord("n"), ord("d")):
            idx = (idx + 1) % len(images)
            load(idx)
        elif key in (ord("p"), ord("a")):
            idx = (idx - 1) % len(images)
            load(idx)

    cv2.destroyAllWindows()


if __name__ == "__main__":
    main()
