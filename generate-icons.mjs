#!/usr/bin/env node
import sharp from 'sharp';
import { readFileSync, writeFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const svgBuffer = readFileSync(join(__dirname, 'icon.svg'));
const iconsDir = join(__dirname, 'src-tauri', 'icons');

const sizes = [32, 128, 256, 512, 1024];

async function generateIcons() {
  console.log('Generating icons from SVG...');
  
  // Generate PNG files at various sizes
  for (const size of sizes) {
    await sharp(svgBuffer)
      .resize(size, size)
      .png()
      .toFile(join(iconsDir, `${size}x${size}.png`));
    console.log(`✓ ${size}x${size}.png`);
    
    // 2x version for 128px
    if (size === 128) {
      await sharp(svgBuffer)
        .resize(256, 256)
        .png()
        .toFile(join(iconsDir, '128x128@2x.png'));
      console.log('✓ 128x128@2x.png');
    }
  }
  
  // Base icon.png
  await sharp(svgBuffer)
    .resize(512, 512)
    .png()
    .toFile(join(iconsDir, 'icon.png'));
  console.log('✓ icon.png');
  
  // Windows Store icons
  const storeLogos = [30, 44, 71, 89, 107, 142, 150, 284, 310];
  for (const size of storeLogos) {
    await sharp(svgBuffer)
      .resize(size, size)
      .png()
      .toFile(join(iconsDir, `Square${size}x${size}Logo.png`));
    console.log(`✓ Square${size}x${size}Logo.png`);
  }
  
  await sharp(svgBuffer)
    .resize(50, 50)
    .png()
    .toFile(join(iconsDir, 'StoreLogo.png'));
  console.log('✓ StoreLogo.png');
  
  console.log('\n✅ All icons generated!');
  console.log('\nNote: .ico and .icns files need platform-specific tools.');
  console.log('Windows: Use png2ico or similar tool to create icon.ico from the PNGs');
  console.log('macOS: Use iconutil or similar tool to create icon.icns');
}

generateIcons().catch(console.error);
