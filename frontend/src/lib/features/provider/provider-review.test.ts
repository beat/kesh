// Story 39-11d2 (AC-2, AC-3) — la date d'un écart ou d'une ligne « à examiner » : celle de sa
// transaction, sinon de sa première écriture, sinon `details.date` (l'écart de continuité Wise,
// `balance_gap`, n'a ni transaction ni écriture).

import { describe, it, expect } from 'vitest';
import { findingDate } from './provider-review';

const tx = { id: 1, bankAccountId: 1, date: '2020-03-10', amount: '1.00', currency: 'CHF', providerType: null, name: null, transactionId: null, counterpartyEmail: null };
const entry = { journalEntryId: 2, entryNumber: 3, entryDate: '2020-04-01', description: '', ids: [] };

describe('findingDate (Story 39-11d2)', () => {
	it('transaction, puis écriture, puis details.date ; rien sans elles', () => {
		expect(findingDate({ code: 'x', transaction: tx, entries: [entry], details: { date: '2020-05-01' } })).toBe('10.03.2020');
		expect(findingDate({ code: 'x', entries: [entry], details: { date: '2020-05-01' } })).toBe('01.04.2020');
		expect(findingDate({ code: 'balance_gap', details: { date: '2020-05-01' } })).toBe('01.05.2020');
		expect(findingDate({ code: 'x', details: { date: 7 } })).toBe('');
		expect(findingDate({ code: 'x' })).toBe('');
	});
});
