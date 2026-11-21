import { inflateSync } from 'node:zlib';

/**
 * Lecture du texte d'un PDF produit par Kesh (`printpdf`), pour les E2E.
 *
 * Déplacé de `invoice-qr-caracteres.spec.ts` (Story 47-4) : la QR-facture et les
 * exports de rapports écrivent leur texte en `Tj` hexadécimal, dans des flux
 * compressés (Flate).
 */

/** Les flux du PDF, décompressés au besoin, en minuscules. */
export function contenuDuPdf(pdf: Buffer): string {
	const brut = pdf.toString('latin1');
	const flux = /stream\r?\n([\s\S]*?)endstream/g;
	let sortie = '';
	for (const m of brut.matchAll(flux)) {
		const octets = Buffer.from(m[1], 'latin1');
		try {
			sortie += inflateSync(octets).toString('latin1');
		} catch {
			sortie += m[1];
		}
	}
	return sortie.toLowerCase();
}

/** Un texte tel qu'il s'écrit dans un `Tj` hexadécimal, Latin-1 valant WinAnsi ici. */
export function hexWinAnsi(texte: string): string {
	return Buffer.from(texte, 'latin1').toString('hex');
}

/**
 * Le texte d'un `Tj` entier, tel que `contenuDuPdf` le rend : `<…> tj`. Chercher
 * cette forme, et non le seul hexadécimal, rend un **jeton entier** : `1020` ne
 * se trouve pas dans le `Tj` de `11020.00`.
 */
export function tjDe(texte: string): string {
	return `<${hexWinAnsi(texte)}> tj`;
}
