import { test, expect, type APIRequestContext } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Tests E2E — compte miroir au plan comptable (Story 47-2, AC-8).
 *
 * Les comptes sont créés par l'API, sans parent, à numéros uniques ; aucun
 * miroir n'est posé sur un compte du seed. Les liens sont retirés en `finally`
 * (une source archivée est d'abord réactivée : un compte archivé ne se modifie
 * pas).
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

interface Compte {
	id: number;
	number: string;
	name: string;
	accountType: string;
	role: string | null;
	postable: boolean;
	active: boolean;
	version: number;
	mirrorAccountId?: number;
}

async function loginAndGoToAccounts(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
	await page.goto('/accounts');
	await expect(page).toHaveURL('/accounts');
}

async function creer(
	ctx: APIRequestContext,
	number: string,
	accountType: string,
	postable: boolean,
): Promise<Compte> {
	const res = await ctx.post('/api/v1/accounts', {
		data: { number, name: `Miroir E2E ${number}`, accountType, postable },
	});
	expect(res.ok(), `création de ${number} : ${res.status()}`).toBeTruthy();
	return (await res.json()) as Compte;
}

async function lire(ctx: APIRequestContext, id: number): Promise<Compte> {
	const res = await ctx.get('/api/v1/accounts?includeArchived=true');
	const list = (await res.json()) as Compte[];
	return list.find((a) => a.id === id)!;
}

/** `PUT` du compte tel qu'en base, plus `extra`. */
async function modifier(ctx: APIRequestContext, id: number, extra: Record<string, unknown>) {
	const a = await lire(ctx, id);
	const res = await ctx.put(`/api/v1/accounts/${id}`, {
		data: {
			name: a.name,
			accountType: a.accountType,
			role: a.role,
			postable: a.postable,
			version: a.version,
			...extra,
		},
	});
	expect(res.ok(), `modification de ${a.number} : ${res.status()}`).toBeTruthy();
}

/** Retire le lien d'une source, la réactivant d'abord si elle est archivée. */
async function nettoyer(ctx: APIRequestContext, source: Compte | undefined) {
	if (!source) return;
	let a = await lire(ctx, source.id);
	if (!a.active) {
		await ctx.put(`/api/v1/accounts/${a.id}/reactivate`, { data: { version: a.version } });
		a = await lire(ctx, source.id);
	}
	if (a.mirrorAccountId !== undefined) {
		await modifier(ctx, a.id, { mirrorAccountId: null });
	}
}

test('choisir un compte miroir, le voir, refuser puis accepter l’archivage du miroir', async ({
	page,
}) => {
	await loginAndGoToAccounts(page);
	const ctx = await authedApiContext(page);
	const suffixe = Date.now().toString().slice(-5);
	let source: Compte | undefined;
	try {
		source = await creer(ctx, `M${suffixe}S`, 'Liability', true);
		const miroir = await creer(ctx, `M${suffixe}T`, 'Asset', false);
		await page.reload();

		// Le choisir à l'écran, enregistrer.
		await page.getByTestId(`account-row-${source.number}-edit-button`).click();
		const select = page.getByTestId('account-edit-dialog-mirror');
		await select.selectOption(String(miroir.id));
		await page.getByTestId('account-edit-dialog-submit').click();
		await expect(page.getByTestId('account-edit-dialog-submit')).toHaveCount(0);
		expect((await lire(ctx, source.id)).mirrorAccountId).toBe(miroir.id);

		// Rouvrir : le `<select>` le montre.
		await page.getByTestId(`account-row-${source.number}-edit-button`).click();
		await expect(page.getByTestId('account-edit-dialog-mirror')).toHaveValue(String(miroir.id));
		await page.getByTestId('account-edit-dialog-cancel').click();

		// Ouvrir le miroir : la phrase « compte miroir de … », sans `<select>`.
		await page.getByTestId(`account-row-${miroir.number}-edit-button`).click();
		await expect(page.getByTestId('account-edit-dialog-mirror-of')).toContainText(source.number);
		await expect(page.getByTestId('account-edit-dialog-mirror')).toHaveCount(0);
		await page.getByTestId('account-edit-dialog-cancel').click();

		// Archiver le miroir : refusé, la source nommée, rien à confirmer.
		await page.getByTestId(`account-row-${miroir.number}-archive-button`).click();
		await page.getByTestId('account-archive-dialog-confirm').click();
		await expect(page.getByTestId('account-archive-mirror')).toContainText(source.number);
		await expect(page.getByTestId('account-archive-dialog-confirm')).toHaveCount(0);
		await expect(page.getByTestId('account-archive-dialog-anyway')).toHaveCount(0);
		await page.getByTestId('account-archive-dialog-cancel').click();
		await expect(page.getByTestId(`account-row-${miroir.number}`)).toHaveCount(1);

		// Retirer le miroir (*Aucun*), puis archiver : accepté.
		await page.getByTestId(`account-row-${source.number}-edit-button`).click();
		await page.getByTestId('account-edit-dialog-mirror').selectOption('');
		await page.getByTestId('account-edit-dialog-submit').click();
		await expect(page.getByTestId('account-edit-dialog-submit')).toHaveCount(0);
		expect((await lire(ctx, source.id)).mirrorAccountId).toBeUndefined();
		await page.getByTestId(`account-row-${miroir.number}-archive-button`).click();
		await page.getByTestId('account-archive-dialog-confirm').click();
		await expect(page.getByTestId(`account-row-${miroir.number}`)).toHaveCount(0);
	} finally {
		await nettoyer(ctx, source);
		await disposeContextSafe(ctx);
	}
});

test('source archivée : l’archivage du miroir dit de la réactiver, puis réactivation, retrait, archivage', async ({
	page,
}) => {
	await loginAndGoToAccounts(page);
	const ctx = await authedApiContext(page);
	const suffixe = Date.now().toString().slice(-5);
	let source: Compte | undefined;
	try {
		source = await creer(ctx, `N${suffixe}S`, 'Expense', true);
		const miroir = await creer(ctx, `N${suffixe}T`, 'Revenue', false);
		await modifier(ctx, source.id, { mirrorAccountId: miroir.id });
		const s = await lire(ctx, source.id);
		const arch = await ctx.put(`/api/v1/accounts/${s.id}/archive`, { data: { version: s.version } });
		expect(arch.ok(), `archivage de la source : ${arch.status()}`).toBeTruthy();
		await page.reload();

		await page.getByTestId(`account-row-${miroir.number}-archive-button`).click();
		await page.getByTestId('account-archive-dialog-confirm').click();
		const refus = page.getByTestId('account-archive-mirror');
		await expect(refus).toContainText(source.number);
		await expect(refus).toContainText(/réactivez/i);
		await expect(page.getByTestId('account-archive-dialog-confirm')).toHaveCount(0);
		await page.getByTestId('account-archive-dialog-cancel').click();

		// Réactiver la source, retirer son lien, puis archiver le miroir.
		await page.getByTestId('account-show-archived-toggle').check();
		await page.getByTestId(`account-row-${source.number}-reactivate-button`).click();
		await expect(page.getByTestId(`account-row-${source.number}-edit-button`)).toBeVisible();
		await page.getByTestId(`account-row-${source.number}-edit-button`).click();
		await page.getByTestId('account-edit-dialog-mirror').selectOption('');
		await page.getByTestId('account-edit-dialog-submit').click();
		await expect(page.getByTestId('account-edit-dialog-submit')).toHaveCount(0);
		await page.getByTestId(`account-row-${miroir.number}-archive-button`).click();
		await page.getByTestId('account-archive-dialog-confirm').click();
		await expect(page.getByTestId(`account-row-${miroir.number}-reactivate-button`)).toBeVisible();
	} finally {
		await nettoyer(ctx, source);
		await disposeContextSafe(ctx);
	}
});
