"""
Generate icons for a Tauri Application
----------------------
By Ibai Farina
For correct functionality, the base logo should be 512x512 or larger (keeping the aspect ratio).
"""

__title__ = 'Tauri Icon Generator'
__author__ = 'Ibai Farina'
__license__ = 'MIT'
__version__ = '0.1.0'

import os
import sys

os.system("cls" if os.name == "nt" else "clear")

try:
    from PIL import Image
    import icnsutil
except ImportError:
    print("Error while importing the required libraries to run this app.")
    print("Make sure you have installed:")
    print("- PIL (https://pypi.org/project/Pillow/) : manipulate images")
    print("- icnsutil (https://pypi.org/project/icnsutil/) : create the .icns macOS icon file")
    sys.exit(-1)

icon_guides = {
    "32x32": 32,
    "128x128": 128,
    "128x128@2x": 256,
    "icon": 512,
    "Square30x30Logo": 30,
    "Square44x44Logo": 44,
    "Square71x71Logo": 71,
    "Square89x89Logo": 89,
    "Square107x107Logo": 107,
    "Square142x142Logo": 142,
    "Square150x150Logo": 150,
    "Square284x284Logo": 284,
    "Square310x310Logo": 310,
    "StoreLogo": 50,
}

base_image_path = input("Enter the 1024x1024 base icon path: ")

try:
    base_image = Image.open(base_image_path).convert("RGBA")
except FileNotFoundError:
    print("Specified file doesn't exist!")
    sys.exit(-1)

if not os.path.exists("icons"):
    print("Creating `icons` folder")
    os.mkdir("icons")

# All png icons (saved as RGBA)
for filename, size in icon_guides.items():
    resized = base_image.resize((size, size), Image.LANCZOS)
    resized.save(os.path.join("icons", filename + ".png"))

# .ico file — include standard multi-size set
ico_sizes = [16, 32, 48, 64, 128, 256]
ico_images = [base_image.resize((s, s), Image.LANCZOS) for s in ico_sizes]
ico_images[0].save(
    os.path.join("icons", "icon.ico"),
    format="ICO",
    append_images=ico_images[1:],
    sizes=[(s, s) for s in ico_sizes],
)

# macOS icns icons
icns = icnsutil.IcnsFile()
icns.add_media(file=base_image_path)
icns.add_media(file=os.path.join("icons", "icon.png"))
icns.add_media(file=os.path.join("icons", "128x128@2x.png"))
icns.write(os.path.join("icons", "icon.icns"))

print("Done!")