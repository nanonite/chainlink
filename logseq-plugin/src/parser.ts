export type ChainlinkStatus = 'open' | 'closed';

export interface ChainlinkCard {
  id: number;
  title: string;
  priority: string;
  labels: string[];
  time: string;
  status: ChainlinkStatus;
  pageRef: string;
}

const ISSUE_LINK_RE = /\[\[chainlink\/issues\/(\d+)\]\]/i;

type Section = 'open' | 'closed' | 'other';

export function parseDashboard(markdown: string): ChainlinkCard[] {
  const cards: ChainlinkCard[] = [];
  let section: Section = 'other';
  let inTable = false;

  for (const rawLine of markdown.split(/\r?\n/)) {
    const line = rawLine.trim();
    const heading = line.match(/^##\s+(.+)$/);
    if (heading) {
      section = normalizeSection(heading[1]);
      inTable = false;
      continue;
    }

    if (section === 'other') {
      continue;
    }

    if (line.startsWith('|')) {
      if (/^\|\s*-+/.test(line) || /\|\s*Priority\s*\|/i.test(line)) {
        inTable = true;
        continue;
      }
      if (inTable) {
        const card = parseIssueRow(line, section);
        if (card) {
          cards.push(card);
        }
      }
    } else if (inTable && line !== '') {
      inTable = false;
    }
  }

  return cards;
}

function normalizeSection(value: string): Section {
  const normalized = value.toLowerCase();
  if (normalized.includes('open issue')) {
    return 'open';
  }
  if (normalized.includes('closed issue')) {
    return 'closed';
  }
  return 'other';
}

function parseIssueRow(line: string, status: ChainlinkStatus): ChainlinkCard | null {
  const cells = splitMarkdownRow(line);
  if (cells.length < 4) {
    return null;
  }

  const [priority, issueCell, labelsCell, time] = cells;
  const link = issueCell.match(ISSUE_LINK_RE);
  if (!link) {
    return null;
  }

  const id = Number.parseInt(link[1], 10);
  const title = issueCell.replace(ISSUE_LINK_RE, '').trim();

  return {
    id,
    title: title || `Issue #${id}`,
    priority: cleanCell(priority),
    labels: parseLabels(labelsCell),
    time: cleanCell(time),
    status,
    pageRef: `chainlink/issues/${link[1]}`
  };
}

function splitMarkdownRow(line: string): string[] {
  const cells: string[] = [];
  let current = '';
  let escaped = false;
  const trimmed = line.replace(/^\|/, '').replace(/\|$/, '');

  for (const char of trimmed) {
    if (escaped) {
      current += char;
      escaped = false;
      continue;
    }
    if (char === '\\') {
      escaped = true;
      continue;
    }
    if (char === '|') {
      cells.push(cleanCell(current));
      current = '';
      continue;
    }
    current += char;
  }
  cells.push(cleanCell(current));
  return cells;
}

function parseLabels(value: string): string[] {
  const cleaned = cleanCell(value);
  if (!cleaned || cleaned === '-' || cleaned === '—') {
    return [];
  }
  return cleaned
    .split(',')
    .map((label) => label.trim())
    .filter(Boolean);
}

function cleanCell(value: string): string {
  return value.replace(/\\\|/g, '|').trim();
}
