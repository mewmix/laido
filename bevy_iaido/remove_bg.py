from PIL import Image
import sys

def remove_background(input_path, output_path):
    print(f"Processing {input_path}...")
    try:
        img = Image.open(input_path).convert("RGBA")
        datas = img.getdata()

        new_data = []
        # Sample background color from top-left (0,0)
        bg_color = img.getpixel((0, 0))
        # Add a tolerance
        threshold = 30

        for item in datas:
            # Check difference (Euclidean distance or sum of diffs)
            diff = sum([abs(c - b) for c, b in zip(item[:3], bg_color[:3])])
            if diff < threshold:
                new_data.append((255, 255, 255, 0)) # Transparent
            else:
                new_data.append(item)

        img.putdata(new_data)
        img.save(output_path, "PNG")
        print(f"Saved to {output_path}")
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    remove_background("assets/atlas/swordsman_laido_atlas2.png", "assets/atlas/swordsman_laido_atlas2_transparent.png")
