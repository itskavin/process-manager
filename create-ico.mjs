#!/usr/bin/env node
import sharp from 'sharp';
import { readFileSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// ICO format header writer
function createIco(pngs) {
  // ICO header
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); // Reserved
  header.writeUInt16LE(1, 2); // Type: ICO
  header.writeUInt16LE(pngs.length, 4); // Number of images
  
  // Directory entries
  const dirEntries = Buffer.alloc(16 * pngs.length);
  let offset = 6 + (16 * pngs.length);
  
  pngs.forEach((png, i) => {
    const entry = dirEntries.slice(i * 16, (i + 1) * 16);
    entry.writeUInt8(png.size, 0); // Width
    entry.writeUInt8(png.size, 1); // Height
    entry.writeUInt8(0, 2); // Color palette
    entry.writeUInt8(0, 3); // Reserved
    entry.writeUInt16LE(1, 4); // Color planes
    entry.writeUInt16LE(32, 6); // Bits per pixel
    entry.writeUInt32LE(png.data.length, 8); // Image size
    entry.writeUInt32LE(offset, 12); // Image offset
    offset += png.data.length;
  });
  
  return Buffer.concat([header, dirEntries, ...pngs.map(p => p.data)]);
}

async function main() {
  const sizes = [256, 128, 64, 48, 32, 16];
  const pngs = [];
  
  console.log('Creating ICO file from thunder icon...');
  
  const svgBuffer = readFileSync(join(__dirname, 'icon.svg'));
  
  for (const size of sizes) {
    const pngData = await sharp(svgBuffer)
      .resize(size, size)
      .png()
      .toBuffer();
    
    pngs.push({ size: size === 256 ? 0 : size, data: pngData });
    console.log(`✓ ${size}x${size}`);
  }
  
  const ico = createIco(pngs);
  writeFileSync(join(__dirname, 'src-tauri', 'icons', 'icon.ico'), ico);
  console.log('✅ icon.ico created');
}

main().catch(console.error);
