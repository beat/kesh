import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	createAndValidateInvoiceViaApi,
	ensurePrimaryBankAccountViaApi,
} from './helpers/api-fixtures';
import { contenuDuPdf, tjDe } from './helpers/pdf';

/**
 * Tests E2E — une QR-facture dont le débiteur porte « & » (Story 33-12a, #941).
 *
 * `Müller & Söhne` est au jeu de caractères de la directive (§ 4.1.1) : le PDF se
 * produit, là où le téléchargement du PDF était refusé pour un nom réel. Seul un
 * E2E voit le nom traverser la route jusqu'au fichier.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('un débiteur au nom portant « & » reçoit sa QR-facture', async ({ page }) => {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');

	await ensurePrimaryBankAccountViaApi(page);
	const nom = `Müller & Söhne ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
	const contactId = await createContactWithAddressViaApi(page, nom);
	const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

	const ctx = await authedApiContext(page);
	try {
		const pdf = await ctx.get(`/api/v1/invoices/${invoiceId}/pdf`);
		expect(pdf.status(), await pdf.text()).toBe(200);
		expect(pdf.headers()['content-type']).toContain('application/pdf');
		const octets = await pdf.body();
		expect(octets.subarray(0, 5).toString()).toBe('%PDF-');

		// Le nom traverse jusqu'au papier : au corps et en partie paiement, le NPA et
		// la localité aussi. Au reçu, ce nom long se coupe. La fixture crée une personne
		// de prénom « Pia » : Kesh compose son nom affiché avec.
		const contenu = contenuDuPdf(octets);
		const affiche = tjDe(`Pia ${nom}`);
		expect(contenu.split(affiche).length - 1).toBe(2);
		expect(contenu).toContain(tjDe('9400 Rorschach'));
	} finally {
		await disposeContextSafe(ctx);
	}
});

/**
 * Story 35-2a (#955, AC12) — un caractère que la QR-facture refuse : l'écran
 * montre le message du serveur, qui nomme le champ et le caractère, et non plus
 * la phrase fixe « La facture n'est pas prête… ». Seul un E2E voit le message
 * traverser la route jusqu'à l'écran.
 */
test('un caractère refusé : le message nomme le champ et le caractère', async ({ page }) => {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');

	await ensurePrimaryBankAccountViaApi(page);
	const nom = `Ve-vey ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
	const contactId = await createContactWithAddressViaApi(
		page,
		nom,
		undefined,
		undefined,
		undefined,
		'Ve–vey',
	);
	const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

	await page.goto(`/invoices/${invoiceId}`);
	await page.getByRole('button', { name: /Télécharger.*PDF/i }).click();
	const message = page.getByText(/localité du client/);
	await expect(message.first()).toBeVisible({ timeout: 5000 });
	await expect(message.first()).toContainText('U+2013');
	await expect(page.getByText(/pas prête pour la génération PDF/)).toHaveCount(0);
});
