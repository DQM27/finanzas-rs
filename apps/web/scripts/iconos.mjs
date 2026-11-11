// Genera los íconos de la PWA en public/iconos, sin dependencias.
//
//   node scripts/iconos.mjs
//
// El dibujo: una moneda con el signo de colón (₡) en blanco sobre el acento
// de la paleta Día (#2959AA). Se dibuja con figuras geométricas y se suaviza
// con 4 × 4 muestras por píxel. La versión «maskable» deja el dibujo dentro
// de la zona segura (80 % del centro) para que Android pueda recortarla.

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { deflateSync } from 'node:zlib';

const FONDO = [0x29, 0x59, 0xaa];
const TINTA = [0xff, 0xff, 0xff];
const MUESTRAS = 4;

/** ¿El punto (x, y), en coordenadas de 0 a 1, cae en la tinta? */
function tinta(x, y, escala) {
  // Se trabaja centrado y normalizado a la zona del dibujo. La «C» abre a la
  // derecha, así que el dibujo se corre un poco para que se vea centrado.
  const u = (x - 0.5) / escala - 0.04;
  const v = (y - 0.5) / escala;
  const r = Math.hypot(u, v);

  // La «C»: un anillo abierto a la derecha.
  const angulo = Math.atan2(v, u);
  const enAnillo = r > 0.2 && r < 0.3 && Math.abs(angulo) > 0.75;

  // Las dos barras del colón, un poco inclinadas, que cruzan la «C».
  const barra = (centro) => {
    const desplazado = u - centro - v * 0.18;
    return Math.abs(desplazado) < 0.028 && Math.abs(v) < 0.36;
  };

  return enAnillo || barra(-0.045) || barra(0.075);
}

function png(lado, escala) {
  const filas = [];
  for (let py = 0; py < lado; py++) {
    const fila = Buffer.alloc(1 + lado * 4);
    fila[0] = 0; // sin filtro
    for (let px = 0; px < lado; px++) {
      let cubierto = 0;
      for (let sy = 0; sy < MUESTRAS; sy++) {
        for (let sx = 0; sx < MUESTRAS; sx++) {
          const x = (px + (sx + 0.5) / MUESTRAS) / lado;
          const y = (py + (sy + 0.5) / MUESTRAS) / lado;
          if (tinta(x, y, escala)) cubierto++;
        }
      }
      const a = cubierto / (MUESTRAS * MUESTRAS);
      const o = 1 + px * 4;
      for (let c = 0; c < 3; c++) fila[o + c] = Math.round(FONDO[c] * (1 - a) + TINTA[c] * a);
      fila[o + 3] = 255;
    }
    filas.push(fila);
  }
  return codificar(lado, Buffer.concat(filas));
}

// ---------------------------------------------------------------- PNG

const TABLA_CRC = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});

function crc(datos) {
  let c = 0xffffffff;
  for (const b of datos) c = TABLA_CRC[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function bloque(tipo, datos) {
  const largo = Buffer.alloc(4);
  largo.writeUInt32BE(datos.length);
  const cuerpo = Buffer.concat([Buffer.from(tipo, 'ascii'), datos]);
  const suma = Buffer.alloc(4);
  suma.writeUInt32BE(crc(cuerpo));
  return Buffer.concat([largo, cuerpo, suma]);
}

function codificar(lado, crudo) {
  const cabecera = Buffer.alloc(13);
  cabecera.writeUInt32BE(lado, 0);
  cabecera.writeUInt32BE(lado, 4);
  cabecera[8] = 8; // bits por canal
  cabecera[9] = 6; // RGBA
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    bloque('IHDR', cabecera),
    bloque('IDAT', deflateSync(crudo, { level: 9 })),
    bloque('IEND', Buffer.alloc(0)),
  ]);
}

// ---------------------------------------------------------------- salida

const destino = join(dirname(fileURLToPath(import.meta.url)), '..', 'public', 'iconos');
mkdirSync(destino, { recursive: true });

const iconos = [
  ['icono-192.png', 192, 1],
  ['icono-512.png', 512, 1],
  ['icono-maskable-512.png', 512, 0.8],
  ['favicon-32.png', 32, 1.15],
];

for (const [nombre, lado, escala] of iconos) {
  writeFileSync(join(destino, nombre), png(lado, escala));
  console.log(`${nombre} (${lado} px)`);
}

// El ícono de iPhone va adentro de index.html. Todo el dominio está detrás de
// Cloudflare Access, y el iPhone pide el ícono sin la sesión: si fuera un
// archivo aparte, Access lo bloquearía y el iPhone pondría una letra.
const indice = join(dirname(fileURLToPath(import.meta.url)), '..', 'src', 'index.html');
const html = readFileSync(indice, 'utf8');
const datos = `data:image/png;base64,${png(180, 1).toString('base64')}`;
const nuevo = html.replace(
  /<link rel="apple-touch-icon" href="[^"]*">/,
  `<link rel="apple-touch-icon" href="${datos}">`,
);
if (nuevo === html && !html.includes(datos)) throw new Error('no encontré el apple-touch-icon en index.html');
writeFileSync(indice, nuevo);
console.log('apple-touch-icon embebido en src/index.html');
