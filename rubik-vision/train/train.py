import argparse
import random
from pathlib import Path

import yaml
from ultralytics import YOLO

ROOT_DIR = Path(__file__).parent.parent.parent / "data" / "vision"
IMAGES_DIR = ROOT_DIR / "images"
LABELS_DIR = ROOT_DIR / "labels"
IMG_EXTENSIONS = (".jpg", ".jpeg", ".png", ".bmp", ".tiff", ".tif")


def get_labeled_images():
    images = []
    for ext in IMG_EXTENSIONS:
        images.extend(IMAGES_DIR.glob(f"*{ext}"))
        images.extend(IMAGES_DIR.glob(f"*{ext.upper()}"))
    return sorted(p for p in set(images) if (LABELS_DIR / p.stem).with_suffix(".txt").exists())


def create_dataset_yaml(train_paths, val_paths, out_dir):
    train_txt = out_dir / "train.txt"
    val_txt = out_dir / "val.txt"
    train_txt.write_text("\n".join(str(p.resolve()) for p in train_paths))
    val_txt.write_text("\n".join(str(p.resolve()) for p in val_paths))

    cfg = {
        "train": str(train_txt.resolve()),
        "val": str(val_txt.resolve()),
        "nc": 1,
        "names": ["face"],
        # 4 keypoints, each with (x, y, visibility)
        "kpt_shape": [4, 3],
    }
    yaml_path = out_dir / "dataset.yaml"
    yaml_path.write_text(yaml.dump(cfg, default_flow_style=False))
    return yaml_path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--epochs", type=int, default=200)
    parser.add_argument("--imgsz", type=int, default=640)
    parser.add_argument("--batch", type=int, default=16)
    parser.add_argument("--val-split", type=float, default=0.2, metavar="FRAC")
    args = parser.parse_args()

    images = get_labeled_images()
    if not images:
        print(f"No labeled images found in {IMAGES_DIR}")
        return

    rng = random.Random(42)
    shuffled = images.copy()
    rng.shuffle(shuffled)
    n_val = max(1, int(len(shuffled) * args.val_split))
    val = shuffled[:n_val]
    train = shuffled[n_val:]
    print(f"Dataset: {len(train)} train, {len(val)} val ({len(images)} total)")

    out_dir = Path(__file__).parent / "dataset"
    out_dir.mkdir(exist_ok=True)
    yaml_path = create_dataset_yaml(train, val, out_dir)

    model = YOLO("yolo26n-pose.pt")
    model.train(
        data=str(yaml_path),
        epochs=args.epochs,
        imgsz=args.imgsz,
        batch=args.batch,
        name="rubik",
        project="rubik",
        perspective=0.001,
        scale=0.3,
        degrees=15.0,
        translate=0.2,
        hsv_h=0.02,
        hsv_s=0.5,
        hsv_v=0.3,
        pose=30.0,
        fliplr=0.0,
        flipud=0.0,
    )


if __name__ == "__main__":
    main()
