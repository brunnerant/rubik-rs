import cv2
from pathlib import Path

IMAGES_DIR = Path(__file__).parent.parent.parent / "data" / "vision" / "images"
OUTPUT_SIZE = 512
WIN_NAME = "Capture"


def next_filename():
    existing = sorted(IMAGES_DIR.glob("*.png"))
    if not existing:
        return IMAGES_DIR / "01.png"
    last = int(existing[-1].stem)
    return IMAGES_DIR / f"{last + 1:02d}.png"


def main():
    IMAGES_DIR.mkdir(parents=True, exist_ok=True)

    cap = cv2.VideoCapture(0)
    if not cap.isOpened():
        print("Cannot open webcam")
        return

    print("Space: capture | q: quit")
    cv2.namedWindow(WIN_NAME, cv2.WINDOW_NORMAL)

    while True:
        ret, frame = cap.read()
        if not ret:
            break

        h, w = frame.shape[:2]
        side = min(h, w)
        crop = frame[(h - side) // 2:(h + side) // 2, (w - side) // 2:(w + side) // 2]
        resized = cv2.resize(crop, (OUTPUT_SIZE, OUTPUT_SIZE), interpolation=cv2.INTER_AREA)

        cv2.imshow(WIN_NAME, resized)
        key = cv2.waitKey(20) & 0xFF

        if key == ord("q"):
            break
        elif key == ord(" "):
            path = next_filename()
            cv2.imwrite(str(path), resized)
            print(f"Saved {path}")

    cap.release()
    cv2.destroyAllWindows()


if __name__ == "__main__":
    main()
