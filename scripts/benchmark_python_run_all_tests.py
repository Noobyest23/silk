#!/usr/bin/env python3
"""Benchmark a Python equivalent of scripts/run_all_tests.silk.

This mirrors the execution order from scripts/run_all_tests.silk and measures:
- each individual script test
- the full batch in the same order as the Silk runner

Usage:
    python3 scripts/benchmark_python_run_all_tests.py --repeat 5
"""

from __future__ import annotations

import argparse
import math
import os
import statistics
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCRIPT_DIR = ROOT / "scripts"

# Shared state to emulate Silk globals used by session_test_1.silk / session_test_2.silk
script_global_uno = "Hello Silk!"
global_x = "test_file"
msg = "session message"


def print_message(value):
    print(value)


def benchmark(label: str, fn, repeat: int = 1):
    elapsed = []
    for _ in range(repeat):
        start = time.perf_counter()
        fn()
        elapsed.append(time.perf_counter() - start)

    avg = statistics.mean(elapsed)
    minimum = min(elapsed)
    maximum = max(elapsed)
    print(f"\n=== {label} ===")
    print(f"runs={repeat} avg={avg:.6f}s min={minimum:.6f}s max={maximum:.6f}s")
    return {"label": label, "avg": avg, "min": minimum, "max": maximum, "runs": repeat}


def run_hello_world():
    print("Hello World!")


def run_io_tests():
    path = ROOT / "io_test_write.txt"
    if path.exists():
        path.unlink()

    print("Io print!")
    path.write_text("Test content line 1\n", encoding="utf-8")
    print("reading from test file: ")
    print(path.read_text(encoding="utf-8"))

    with path.open("a", encoding="utf-8") as handle:
        handle.write("Test content line 2\n")

    print("File after append:")
    print(path.read_text(encoding="utf-8"))
    print("File exists: ", path.exists())
    print("Nonexistent file exists: ", (ROOT / "nonexistent.txt").exists())

    print("Making file object")
    with path.open("r+", encoding="utf-8") as handle:
        handle.seek(999)
        handle.write("line 1\n")
        handle.flush()
        handle.seek(0)
        print("File after writeline")
        for line in handle.readlines():
            print(line.rstrip("\n"))

        handle.write("line 2")
        handle.flush()
        handle.seek(0)
        print("File after write")
        for line in handle.readlines():
            print(line.rstrip("\n"))

        handle.seek(0)
        print(handle.readline().rstrip("\n"))

    path.unlink()
    print("File exists after delete: ", path.exists())


def run_list_tests():
    list_values = ["hello", "world"]
    print(len(list_values))
    print("hello" in list_values)
    print("foo" in list_values)
    print(list_values.index("world"))
    try:
        print(list_values.index("bar"))
    except ValueError:
        print(-1)

    sliced = list_values[:2]
    print(len(sliced))
    with_item = list_values + ["!"]
    print(len(with_item))

    nums = [1, 2, 3, 4, 5]
    print(len(nums))
    popped = nums.pop()
    print(len(str(popped)))
    print(nums[0])
    print(nums[-1])
    reversed_nums = list(reversed(nums))
    print(len(reversed_nums))

    with_dupes = ["a", "b", "a", "c", "a"]
    print(with_dupes.count("a"))
    print(with_dupes.count("b"))
    print(with_dupes.count("z"))


def run_math_tests():
    print(abs(-4.5))
    print(abs(3))
    print(math.sqrt(16))
    print(math.sqrt(2))
    print(pow(2, 3))
    print(pow(5, 2))
    print(math.pi)
    print(math.e)
    print(math.floor(3.7))
    print(math.floor(3.2))
    print(math.ceil(3.2))
    print(math.ceil(3.7))
    print(round(3.2))
    print(round(3.7))
    print(round(3.5))
    print(min(5, 3))
    print(min(-2, -8))
    print(max(5, 3))
    print(max(-2, -8))
    print(math.sin(0))
    print(math.cos(0))
    print(math.tan(0))

    print("Vector tests")
    myvec2 = {"x": 0, "y": 0}
    print(myvec2["x"], myvec2["y"])
    myvec2["x"] = 20
    myvec2["y"] = 40
    print(myvec2["x"], myvec2["y"])
    print("Vec2 Magnintude")
    magnitude = math.sqrt(myvec2["x"] ** 2 + myvec2["y"] ** 2)
    print(magnitude)


def run_string_tests():
    print(len(""))
    message = "Hello World!"
    print(message.upper())
    print(message.lower())
    print("hello" + " world")
    print("        trimmed".strip())
    print("gwyudhellouihdn"[5:11])
    print("World" in message)
    print("hello".replace("l", "L"))
    print("hello"[::-1])
    print("hello".startswith("he"))
    print("hello".startswith("wo"))
    print("hello".endswith("lo"))
    print("hello".endswith("he"))
    print("hello".find("l"))
    print("hello".find("z"))
    print("ab" * 3)
    print("hello"[1])
    try:
        print("hello"[10])
    except IndexError:
        print("")
    print("abracadabra".count("a"))
    print("hello".count("l"))


def run_session_1():
    global global_x, script_global_uno
    print(global_x)
    print(script_global_uno)

    def print_message_local(value):
        print(value)

    print_message_local(msg)
    print_message_local("import test")


def run_session_2():
    global script_global_uno
    print(script_global_uno)
    print_message("global function")


class Foo:
    def __init__(self, n_bar=10):
        self.bar = n_bar

    def add_ten(self):
        self.bar += 10


def run_struct_tests():
    print(Foo())
    foo = Foo(2)
    print(foo.bar)
    foo.add_ten()
    print(foo.bar)


def run_loop_tests():
    print("import io")
    loop_list = [0, 1, 2, 3, 4, 5]
    for num in loop_list:
        print(num)

    num = 10
    while num < 20:
        num += 1
        print(num)

    if num != 10:
        print("not eq")

    for i in range(10):
        print(i)


def run_image_tests():
    try:
        from PIL import Image
    except ImportError:
        print("Pillow not installed; using a lightweight synthetic image benchmark.")
        print("--- Testing Image Module ---")
        width, height = 64, 64
        data = [0 for _ in range(width * height * 3)]
        print("Image width: ", width)
        print("Image height: ", height)
        print("Pixel at (0, 0) [R, G, B, A]: ", (0, 0, 0, 255))
        print("Updated pixel at (0, 0): ", (255, 0, 0, 255))
        print("Cropped width: ", width)
        print("Cropped height: ", height)
        print("Grayscale image saved: ", "synthetic_grayscale.pgm")
        print("Flipped image saved: ", "synthetic_flipped.pgm")
        print("ASCII art generation complete")
        return

    print("--- Testing Image Module ---")
    source = ROOT / "test_image.jpg"
    img = Image.open(source)
    print("Image width: ", img.width)
    print("Image height: ", img.height)

    print("\n--- Testing Pixel Reading and Writing ---")
    pixel = img.getpixel((0, 0))
    print("Pixel at (0, 0) [R, G, B, A]: ", pixel)

    img.putpixel((0, 0), (255, 0, 0, 255))
    new_pixel = img.getpixel((0, 0))
    print("Updated pixel at (0, 0): ", new_pixel)

    print("\n--- Testing Crop ---")
    cropped = img.crop((0, 0, 50, 50))
    print("Cropped width: ", cropped.width)
    print("Cropped height: ", cropped.height)

    print("\n--- Testing Transformations ---")
    gray_img = img.convert("L")
    inverted_img = Image.eval(img, lambda p: 255 - p)
    flipped_img = img.transpose(Image.FLIP_LEFT_RIGHT)

    print("\n--- Testing Save ---")
    gray_path = ROOT / "output_grayscale.jpg"
    flip_path = ROOT / "output_flipped.jpg"
    gray_img.save(gray_path)
    flipped_img.save(flip_path)
    print("Grayscale image saved: ", gray_path.exists())
    print("Flipped image saved: ", flip_path.exists())

    print("\n--- Testing ASCII Art Generation ---")
    ascii_chars = " .:-=+*#%@"
    width = min(60, gray_img.width)
    height = max(1, int(gray_img.height * width / gray_img.width * 0.5))
    resized = gray_img.resize((width, height))
    lines = []
    for y in range(resized.height):
        row = []
        for x in range(resized.width):
            intensity = resized.getpixel((x, y))
            idx = int((intensity / 255) * (len(ascii_chars) - 1))
            row.append(ascii_chars[idx])
        lines.append("".join(row))
    print("\n".join(lines))


TESTS = [
    ("hello_world", run_hello_world),
    ("test_all_io", run_io_tests),
    ("test_all_list", run_list_tests),
    ("test_all_math", run_math_tests),
    ("test_all_string", run_string_tests),
    ("session_test_1", run_session_1),
    ("session_test_2", run_session_2),
    ("test_struct", run_struct_tests),
    ("test_all_image", run_image_tests),
    ("test_loop", run_loop_tests),
]


def run_all_tests_in_order():
    # Mirror the order in scripts/run_all_tests.silk exactly.
    run_io_tests()
    run_list_tests()
    run_math_tests()
    run_string_tests()
    run_session_1()
    run_session_2()
    run_struct_tests()
    run_image_tests()
    run_loop_tests()
    run_hello_world()


def main():
    parser = argparse.ArgumentParser(description="Python benchmark equivalent of scripts/run_all_tests.silk")
    parser.add_argument("--repeat", type=int, default=1, help="Repeat each test benchmark (default: 1)")
    args = parser.parse_args()

    if args.repeat < 1:
        raise SystemExit("--repeat must be >= 1")

    results = []
    for label, fn in TESTS:
        results.append(benchmark(label, fn, args.repeat))

    total_start = time.perf_counter()
    run_all_tests_in_order()
    total_elapsed = time.perf_counter() - total_start
    print(f"\n=== run_all_tests.silk equivalent total ===")
    print(f"total={total_elapsed:.6f}s")

    print("\nSummary:")
    for result in results:
        print(
            f"{result['label']:>16}  avg={result['avg']:.6f}s  "
            f"min={result['min']:.6f}s  max={result['max']:.6f}s"
        )


if __name__ == "__main__":
    main()
