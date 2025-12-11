const fs = require('fs');
const path = require('path');

// Simple ICO header and directory entry for a single 256x256 PNG image
function createIco(pngBuffer) {
    const header = Buffer.alloc(6);
    header.writeUInt16LE(0, 0); // Reserved
    header.writeUInt16LE(1, 2); // Type (1 = ICO)
    header.writeUInt16LE(1, 4); // Count (1 image)

    const entry = Buffer.alloc(16);
    entry.writeUInt8(0, 0);   // Width (0 = 256)
    entry.writeUInt8(0, 1);   // Height (0 = 256)
    entry.writeUInt8(0, 2);   // Color count (0 = no palette)
    entry.writeUInt8(0, 3);   // Reserved
    entry.writeUInt16LE(1, 4); // Color planes
    entry.writeUInt16LE(32, 6); // Bits per pixel
    entry.writeUInt32LE(pngBuffer.length, 8); // Size of visual data
    entry.writeUInt32LE(22, 12); // Offset of visual data (6 + 16)

    return Buffer.concat([header, entry, pngBuffer]);
}

const sourcePng = process.argv[2];
const destIco = process.argv[3];

if (!sourcePng || !destIco) {
    console.error("Usage: node generate_ico.js <input.png> <output.ico>");
    process.exit(1);
}

try {
    const pngData = fs.readFileSync(sourcePng);
    const icoData = createIco(pngData);
    fs.writeFileSync(destIco, icoData);
    console.log(`Successfully created ${destIco} from ${sourcePng}`);
} catch (err) {
    console.error("Error:", err);
    process.exit(1);
}
