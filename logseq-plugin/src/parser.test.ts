import { describe, expect, it } from 'vitest';
import { parseDashboard } from './parser';

describe('parseDashboard', () => {
  it('parses open issue rows from the dashboard export', () => {
    const cards = parseDashboard(`## Open Issues

| Priority | Issue | Labels | Time |
|----------|-------|--------|------|
| high | [[chainlink/issues/0003]] Add logseq integration | logseq, ui | 2h 15m |
`);

    expect(cards).toEqual([
      {
        id: 3,
        title: 'Add logseq integration',
        priority: 'high',
        labels: ['logseq', 'ui'],
        time: '2h 15m',
        status: 'open',
        pageRef: 'chainlink/issues/0003'
      }
    ]);
  });

  it('ignores non-issue sections', () => {
    const cards = parseDashboard(`## Active Timers

| Issue | Started |
|-------|---------|
| [[chainlink/issues/0001]] | 2026-05-19 |
`);

    expect(cards).toHaveLength(0);
  });
});
