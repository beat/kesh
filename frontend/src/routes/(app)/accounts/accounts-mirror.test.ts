// Story 47-2 (AC-2, AC-3, AC-6) — le compte miroir au dialogue d'un compte et le
// refus de l'archivage d'un compte miroir. Harnais d'`accounts-page.test.ts`.
//
// ⛔ `isApiError` n'est PAS mocké : une erreur simulée porte `code` ET `status`.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { authState } from '$lib/app/stores/auth.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import { toast } from 'svelte-sonner';

vi.mock('$app/environment', () => ({ browser: true }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, unknown>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
}));

vi.mock('svelte-sonner', () => ({
	toast: { success: vi.fn(), error: vi.fn() },
}));

const fetchAccountsMock = vi.fn<() => Promise<AccountResponse[]>>();
const updateAccountMock = vi.fn();
const createAccountMock = vi.fn();
const archiveAccountMock = vi.fn();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock(),
	updateAccount: (id: number, req: unknown) => updateAccountMock(id, req),
	createAccount: (req: unknown) => createAccountMock(req),
	archiveAccount: (id: number, req: unknown) => archiveAccountMock(id, req),
	reactivateAccount: vi.fn(),
}));
vi.mock('$lib/features/settings/settings.api', () => ({
	fetchCompanyCurrent: () => Promise.reject(new Error('non simulée')),
}));

import Page from './+page.svelte';

function account(overrides: Partial<AccountResponse> & { id: number; number: string }): AccountResponse {
	return {
		companyId: 1,
		name: `Compte ${overrides.number}`,
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...overrides,
	};
}

const SOURCE = account({ id: 1, number: '2270', accountType: 'Liability' });
const MIROIR = account({ id: 2, number: '1179', postable: false });
const AUTRE = account({ id: 3, number: '1178', postable: false });

const MIRROR_INVALID = {
	code: 'MIRROR_ACCOUNT_INVALID',
	message: 'Le compte miroir doit être un compte de présentation.',
	status: 400,
	details: { reason: 'notPresentationAccount' },
};

async function openEdit(number: string) {
	await fireEvent.click(await screen.findByTestId(`account-row-${number}-edit-button`));
}

beforeEach(() => {
	fetchAccountsMock.mockReset();
	updateAccountMock.mockReset();
	createAccountMock.mockReset();
	archiveAccountMock.mockReset();
	vi.mocked(toast.error).mockClear();
	fetchAccountsMock.mockResolvedValue([MIROIR, AUTRE, SOURCE]);
	authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
});

afterEach(async () => {
	await authState.logout();
});

describe('dialogue de modification — compte miroir', () => {
	it('un <select> natif, *Aucun* et les candidats de l’autre côté', async () => {
		render(Page);
		await openEdit('2270');
		const select = (await screen.findByTestId('account-edit-dialog-mirror')) as HTMLSelectElement;
		expect(select.tagName).toBe('SELECT');
		expect(Array.from(select.options).map((o) => o.value)).toEqual(['', '2', '3']);
		expect(select.getAttribute('aria-describedby')).toBe('account-edit-mirror-help');
	});

	it('le champ n’est pas envoyé s’il n’a pas changé', async () => {
		updateAccountMock.mockResolvedValue(SOURCE);
		render(Page);
		await openEdit('2270');
		await fireEvent.click(await screen.findByTestId('account-edit-dialog-submit'));
		await waitFor(() => expect(updateAccountMock).toHaveBeenCalled());
		expect(updateAccountMock.mock.calls[0][1]).not.toHaveProperty('mirrorAccountId');
	});

	it('le miroir choisi est envoyé ; « Aucun » envoie null', async () => {
		updateAccountMock.mockResolvedValue(SOURCE);
		fetchAccountsMock.mockResolvedValue([MIROIR, AUTRE, { ...SOURCE, mirrorAccountId: 2 }]);
		render(Page);
		await openEdit('2270');
		const select = (await screen.findByTestId('account-edit-dialog-mirror')) as HTMLSelectElement;
		expect(select.value).toBe('2');
		await fireEvent.change(select, { target: { value: '' } });
		await fireEvent.click(await screen.findByTestId('account-edit-dialog-submit'));
		await waitFor(() => expect(updateAccountMock).toHaveBeenCalled());
		expect(updateAccountMock.mock.calls[0][1]).toMatchObject({ mirrorAccountId: null });
	});

	it('pour un compte miroir, une phrase qui nomme les sources, sans <select>', async () => {
		fetchAccountsMock.mockResolvedValue([
			MIROIR,
			AUTRE,
			{ ...SOURCE, mirrorAccountId: 2 },
			account({ id: 4, number: '2271', accountType: 'Liability', mirrorAccountId: 2 }),
		]);
		updateAccountMock.mockResolvedValue(MIROIR);
		render(Page);
		await openEdit('1179');
		const phrase = await screen.findByTestId('account-edit-dialog-mirror-of');
		expect(phrase.textContent).toContain('2270, 2271');
		expect(screen.queryByTestId('account-edit-dialog-mirror')).toBeNull();
		await fireEvent.click(await screen.findByTestId('account-edit-dialog-submit'));
		await waitFor(() => expect(updateAccountMock).toHaveBeenCalled());
		expect(updateAccountMock.mock.calls[0][1]).not.toHaveProperty('mirrorAccountId');
	});

	it('le refus MIRROR_ACCOUNT_INVALID s’affiche sous le champ, non au pied', async () => {
		updateAccountMock.mockRejectedValue(MIRROR_INVALID);
		render(Page);
		await openEdit('2270');
		const select = await screen.findByTestId('account-edit-dialog-mirror');
		await fireEvent.change(select, { target: { value: '3' } });
		await fireEvent.click(await screen.findByTestId('account-edit-dialog-submit'));
		const err = await screen.findByTestId('account-edit-dialog-mirror-error');
		expect(err.textContent).toContain(MIRROR_INVALID.message);
		expect(err.getAttribute('role')).toBe('alert');
		expect(select.getAttribute('aria-invalid')).toBe('true');
	});

	it('rôle de fonds propres : le <select> reste, le refus du serveur s’affiche dessous', async () => {
		const capital = account({ id: 5, number: '2800', accountType: 'Liability', role: 'EquityCapital' });
		fetchAccountsMock.mockResolvedValue([MIROIR, AUTRE, capital]);
		updateAccountMock.mockRejectedValue({ ...MIRROR_INVALID, details: { reason: 'equity' } });
		render(Page);
		await openEdit('2800');
		const select = await screen.findByTestId('account-edit-dialog-mirror');
		await fireEvent.change(select, { target: { value: '2' } });
		await fireEvent.click(await screen.findByTestId('account-edit-dialog-submit'));
		expect(await screen.findByTestId('account-edit-dialog-mirror-error')).toBeTruthy();
	});

	it('cas de concurrence : le refus du rejeu de confirmRetype (lien posé entre les deux envois) tombe aussi sous le champ', async () => {
		updateAccountMock
			.mockRejectedValueOnce({
				code: 'ACCOUNT_HAS_ENTRIES',
				message: 'écritures',
				status: 409,
				details: { entryCount: 1, closedFiscalYears: [], fromType: 'Liability', toType: 'Asset' },
			})
			.mockRejectedValueOnce({ ...MIRROR_INVALID, details: { reason: 'accountType' } });
		fetchAccountsMock.mockResolvedValue([MIROIR, AUTRE, { ...SOURCE, mirrorAccountId: 2 }]);
		render(Page);
		await openEdit('2270');
		await fireEvent.click(await screen.findByTestId('account-edit-dialog-submit'));
		await fireEvent.click(await screen.findByTestId('account-retype-dialog-confirm'));
		expect(await screen.findByTestId('account-edit-dialog-mirror-error')).toBeTruthy();
	});
});

describe('dialogue de création — compte miroir', () => {
	it('le miroir choisi est envoyé à la création, rien pour « Aucun »', async () => {
		createAccountMock.mockResolvedValue(SOURCE);
		render(Page);
		await fireEvent.click(await screen.findByTestId('account-create-button'));
		const select = (await screen.findByTestId('account-create-dialog-mirror')) as HTMLSelectElement;
		// Type par défaut : Actif → aucun passif non postable dans ce plan.
		expect(Array.from(select.options).map((o) => o.value)).toEqual(['']);
		await fireEvent.input(screen.getByLabelText('Numéro'), { target: { value: '2272' } });
		await fireEvent.input(screen.getByLabelText('Nom'), { target: { value: 'X' } });
		await fireEvent.click(await screen.findByTestId('account-create-dialog-submit'));
		await waitFor(() => expect(createAccountMock).toHaveBeenCalled());
		expect(createAccountMock.mock.calls[0][0]).not.toHaveProperty('mirrorAccountId');
	});

	it('le refus s’affiche sous le champ de création', async () => {
		createAccountMock.mockRejectedValue(MIRROR_INVALID);
		render(Page);
		await fireEvent.click(await screen.findByTestId('account-create-button'));
		await fireEvent.input(screen.getByLabelText('Numéro'), { target: { value: '2272' } });
		await fireEvent.input(screen.getByLabelText('Nom'), { target: { value: 'X' } });
		await fireEvent.click(await screen.findByTestId('account-create-dialog-submit'));
		expect(await screen.findByTestId('account-create-dialog-mirror-error')).toBeTruthy();
	});
});

describe('archivage d’un compte miroir', () => {
	const IS_MIRROR = {
		code: 'ACCOUNT_IS_MIRROR',
		message: 'Ce compte est le compte miroir de 2270 Compte 2270 : il ne s’archive qu’une fois le lien retiré.',
		status: 409,
		details: { reason: 'archive', sources: [{ id: 1, number: '2270', name: 'Compte 2270', active: true }] },
	};

	it('le refus dans le dialogue, sans toast, sans *Archiver* ni « Archiver quand même »', async () => {
		archiveAccountMock.mockRejectedValue(IS_MIRROR);
		render(Page);
		await fireEvent.click(await screen.findByTestId('account-row-1179-archive-button'));
		await fireEvent.click(await screen.findByTestId('account-archive-dialog-confirm'));
		const msg = await screen.findByTestId('account-archive-mirror');
		expect(msg.textContent).toContain('2270');
		expect(msg.getAttribute('role')).toBe('alert');
		expect(screen.queryByTestId('account-archive-dialog-confirm')).toBeNull();
		expect(screen.queryByTestId('account-archive-dialog-anyway')).toBeNull();
		expect(screen.getByTestId('account-archive-dialog-cancel')).toBeTruthy();
		expect(toast.error).not.toHaveBeenCalled();
	});

	it('rouvrir le dialogue repart sans le refus', async () => {
		archiveAccountMock.mockRejectedValue(IS_MIRROR);
		render(Page);
		await fireEvent.click(await screen.findByTestId('account-row-1179-archive-button'));
		await fireEvent.click(await screen.findByTestId('account-archive-dialog-confirm'));
		await screen.findByTestId('account-archive-mirror');
		await fireEvent.click(screen.getByTestId('account-archive-dialog-cancel'));
		await fireEvent.click(await screen.findByTestId('account-row-1179-archive-button'));
		expect(await screen.findByTestId('account-archive-dialog-confirm')).toBeTruthy();
		expect(screen.queryByTestId('account-archive-mirror')).toBeNull();
	});
});
