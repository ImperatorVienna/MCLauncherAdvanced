#!/usr/bin/env python3
"""Import a small icon set from a locally installed official client JAR.
Usage: python3 scripts/import-game-icons.py /path/to/version.jar
No downloads or writes to the source game directory.
"""
import base64
import hashlib
import json
from pathlib import Path
import sys
import zipfile

root = Path(__file__).resolve().parents[1]
out = root / 'apps/desktop/src/assets/installation-icons'
out.mkdir(parents=True, exist_ok=True)
jar = Path(sys.argv[1])
sources = {}
with zipfile.ZipFile(jar) as archive:
    def texture(name):
        path = f'assets/minecraft/textures/{name}.png'
        data = archive.read(path)
        sources[path] = hashlib.sha256(data).hexdigest()
        return data

    def face(data, transform, extra=''):
        url = base64.b64encode(data).decode()
        return f'<image width="16" height="16" transform="{transform}" href="data:image/png;base64,{url}" {extra}/>'

    blocks = {'grass':'grass_block_side', 'sand':'sand', 'cobblestone':'cobblestone', 'oak':'oak_planks', 'brick':'bricks', 'obsidian':'obsidian', 'crafting_table':'crafting_table_front', 'furnace':'furnace_front'}
    for icon, name in blocks.items():
        side = texture('block/' + name)
        top = texture('block/grass_block_top') if icon == 'grass' else texture('block/' + icon + '_top') if icon in ['crafting_table', 'furnace'] else side
        svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" style="image-rendering:pixelated">'
        svg += '<defs><filter id="grass" color-interpolation-filters="sRGB"><feComponentTransfer><feFuncR type="linear" slope="0.569"/><feFuncG type="linear" slope="0.741"/><feFuncB type="linear" slope="0.349"/></feComponentTransfer></filter></defs>'
        svg += face(top, 'matrix(1 .5 -1 .5 16 0)', 'filter="url(#grass)"' if icon == 'grass' else '')
        for index, matrix in enumerate(['matrix(1 .5 0 1 0 8)', 'matrix(1 -.5 0 1 16 16)']):
            face_texture = texture('block/' + icon + '_side') if index == 1 and icon in ['crafting_table', 'furnace'] else side
            svg += face(face_texture, matrix)
            if icon == 'grass':
                svg += face(texture('block/grass_block_side_overlay'), matrix, 'filter="url(#grass)"')
        svg += '<path d="M0 8 16 16 16 32 0 24Z" fill="#000" opacity=".16"/><path d="M16 16 32 8 32 24 16 32Z" fill="#000" opacity=".32"/></svg>'
        (out / f'{icon}.svg').write_text(svg)
    # Render the official anvil cuboids, retaining the narrow stem silhouette.
    model_path = 'assets/minecraft/models/block/template_anvil.json'
    model_raw = archive.read(model_path)
    sources[model_path] = hashlib.sha256(model_raw).hexdigest()
    model = json.loads(model_raw)
    svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" style="image-rendering:pixelated">'
    def project(x,y,z): return (16+x-z, 16-y+(x+z)/2)
    for element in model['elements']:
        x0,y0,z0 = element['from']; x1,y1,z1 = element['to']
        surfaces = {
            'up': [project(x0,y1,z0),project(x1,y1,z0),project(x0,y1,z1)],
            'south': [project(x0,y1,z1),project(x1,y1,z1),project(x0,y0,z1)],
            'east': [project(x1,y1,z1),project(x1,y1,z0),project(x1,y0,z1)]
        }
        for direction, points in surfaces.items():
            f = element['faces'].get(direction)
            if not f: continue
            u0,v0,u1,v1 = f['uv']
            p,q,r = points
            matrix = f"matrix({(q[0]-p[0])/16} {(q[1]-p[1])/16} {(r[0]-p[0])/16} {(r[1]-p[1])/16} {p[0]} {p[1]})"
            data = texture('block/anvil_top' if f['texture'] == '#top' else 'block/anvil')
            url = base64.b64encode(data).decode()
            # Texture rotation is applied inside the cropped UV tile.
            svg += f'<g transform="{matrix}"><g transform="rotate({f.get("rotation",0)} 8 8)"><svg width="16" height="16" viewBox="{min(u0,u1)} {min(v0,v1)} {abs(u1-u0)} {abs(v1-v0)}" preserveAspectRatio="none"><image width="16" height="16" href="data:image/png;base64,{url}"/></svg></g>'
            shade = {'up':0,'south':.16,'east':.32}[direction]
            svg += f'<rect width="16" height="16" fill="black" opacity="{shade}"/></g>'
    (out / 'anvil.svg').write_text(svg + '</svg>')
    for icon in ['diamond', 'emerald', 'redstone', 'diamond_sword', 'iron_pickaxe', 'bow', 'apple', 'golden_apple', 'ender_pearl', 'enchanted_book', 'coal', 'iron_ingot', 'gold_ingot', 'lapis_lazuli', 'totem_of_undying', 'lava_bucket', 'fishing_rod', 'blaze_rod', 'nether_star', 'wooden_axe', 'saddle']:
        (out / f'{icon}.png').write_bytes(texture('item/' + icon))

(out / 'sources.json').write_text(json.dumps({'gameVersion': jar.stem, 'sourceJarSha256': hashlib.sha256(jar.read_bytes()).hexdigest(), 'texturesSha256': sources, 'note': 'Original Minecraft textures, Mojang/Microsoft. Block SVGs generated locally; grass uses a fixed green tint and faces use display shading. Not an official launcher icon set or an open-source asset license.'}, ensure_ascii=False, indent=2) + '\n')
print(f'Imported 30 icons from {jar.name} into {out}')
