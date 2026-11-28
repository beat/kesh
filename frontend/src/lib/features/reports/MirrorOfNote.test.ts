// Story 47-4 (AC-3) — le composant de la mention « Solde de … », seul.

import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string, fallback: string, args?: Record<string, string | number>) =>
		`${key}|${args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_m, k) => String(args[k] ?? '')) : fallback}`,
}));

import MirrorOfNote from './MirrorOfNote.svelte';
import type { AccountBalance } from './reports.types';

function compte(over: Partial<AccountBalance> = {}): AccountBalance {
	return {
		accountId: 40,
		accountNumber: '1040.1',
		accountName: 'Créances sociales',
		accountType: 'Asset',
		active: true,
		balance: '1200.00',
		role: null,
		...over,
	};
}

/** Période du rapport, pour les liens des sources. */
const periode = { from: '2026-01-01', to: '2026-12-31' };

describe('MirrorOfNote (Story 47-4)', () => {
	it('rien sans `mirrorOf`', () => {
		const { container } = render(MirrorOfNote, { account: compte(), ...periode });
		expect(container.textContent).toBe('');
	});

	it('rien pour un `mirrorOf` vide', () => {
		const { container } = render(MirrorOfNote, { account: compte({ mirrorOf: [] }), ...periode });
		expect(container.textContent).toBe('');
	});

	it('une source : clé `-one`, numéro et nom', () => {
		const { getByTestId } = render(MirrorOfNote, {
			...periode,
			account: compte({
				mirrorOf: [{ accountId: 22, accountNumber: '2270', accountName: 'AVS/AI/APG', balance: '-1200.00' }],
			}),
		});
		expect(getByTestId('mirror-of-1040.1').textContent).toBe('reports-mirror-of-one|Solde de 2270 AVS/AI/APG');
	});

	it('plusieurs sources : clé `-other`, dans l’ordre reçu, séparées par des virgules', () => {
		const { getByTestId } = render(MirrorOfNote, {
			...periode,
			account: compte({
				mirrorOf: [
					{ accountId: 22, accountNumber: '2270', accountName: 'AVS/AI/APG', balance: '-1200.00' },
					{ accountId: 23, accountNumber: '2271', accountName: 'LPP', balance: '-5.00' },
					{ accountId: 24, accountNumber: '2272', accountName: 'LAA', balance: '-1.00' },
				],
			}),
		});
		expect(getByTestId('mirror-of-1040.1').textContent).toBe(
			'reports-mirror-of-other|Soldes de 2270 AVS/AI/APG, 2271 LPP, 2272 LAA',
		);
	});

	it('le numéro de chaque source ouvre son grand livre sur la période (liens de l’Epic 55)', () => {
		const { container } = render(MirrorOfNote, {
			...periode,
			account: compte({
				mirrorOf: [
					{ accountId: 22, accountNumber: '2270', accountName: 'AVS/AI/APG', balance: '-1200.00' },
					{ accountId: 23, accountNumber: '2271', accountName: 'LPP', balance: '-5.00' },
				],
			}),
		});
		const liens = [...container.querySelectorAll('a')];
		expect(liens.map((a) => a.textContent)).toEqual(['2270', '2271']);
		expect(liens.map((a) => a.getAttribute('href'))).toEqual([
			'/reports?tab=general-ledger&accountId=22&from=2026-01-01&to=2026-12-31',
			'/reports?tab=general-ledger&accountId=23&from=2026-01-01&to=2026-12-31',
		]);
	});
});
