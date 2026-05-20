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

  it('parses Logseq task blocks with properties', () => {
    const cards = parseDashboard(`## Open Issues

- TODO [[chainlink/issues/0010]] Build the useful board
  priority:: high
  status:: open
  labels:: logseq, ux
  time:: 45m

## Recently Closed

- DONE [[chainlink/issues/0009]] Ship export
  priority:: medium
  status:: closed
  labels:: rust
  time:: 1h 20m
`);

    expect(cards).toHaveLength(2);
    expect(cards[0]).toMatchObject({
      id: 10,
      title: 'Build the useful board',
      priority: 'high',
      labels: ['logseq', 'ux'],
      time: '45m',
      status: 'open'
    });
    expect(cards[1]).toMatchObject({
      id: 9,
      title: 'Ship export',
      priority: 'medium',
      labels: ['rust'],
      time: '1h 20m',
      status: 'closed'
    });
  });

});
