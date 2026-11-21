// Story 47-2 (revue P1, L3) — un miroir choisi puis écarté par un changement de
// type du formulaire ne reste ni dans l'état lié ni dans le `<select>`.

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import type { AccountResponse } from './accounts.types';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import Harness from './AccountMirrorFieldHarness.svelte';

function account(o: Partial<AccountResponse> & { id: number; number: string }): AccountResponse {
	return {
		companyId: 1,
		name: `Compte ${o.number}`,
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: false,
		letterable: false,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...o,
	};
}

describe('AccountMirrorField — valeur écartée par le type', () => {
	it('vide la valeur liée quand le candidat sort de la liste', async () => {
		const passif = account({ id: 9, number: '2999', accountType: 'Liability' });
		render(Harness, { accounts: [passif] });
		const select = (await screen.findByTestId('h-dialog-mirror')) as HTMLSelectElement;
		await fireEvent.change(select, { target: { value: '9' } });
		expect(screen.getByTestId('h-state').textContent).toBe('9');
		// Type Passif : le passif 2999 est du même côté, il n'est plus candidat.
		await fireEvent.click(screen.getByTestId('h-liability'));
		await waitFor(() => expect(screen.getByTestId('h-state').textContent).toBe(''));
		expect(Array.from(select.options).map((o) => o.value)).toEqual(['']);
	});
});
