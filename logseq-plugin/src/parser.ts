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
const TASK_RE = /^[-*]\s+(TODO|DONE)\s+\[\[chainlink\/issues\/(\d+)\]\]\s*(.*)$/i;
const PROPERTY_RE = /^\s*[-*]?\s*([a-z-]+)::\s*(.*)$/i;

type Section = 'open' | 'closed' | 'other';

export function parseDashboard(markdown: string): ChainlinkCard[] {
  const cards: ChainlinkCard[] = [];
  let section: Section = 'other';
  let inTable = false;
  let currentCard: ChainlinkCard | null = null;

  const flush = () => {
    if (currentCard) {
      cards.push(currentCard);
      currentCard = null;
    }
  };

  for (const rawLine of markdown.split(/\r?\n/)) {
    const line = rawLine.trimEnd();
    const normalizedLine = line.trim();
    const heading = normalizedLine.match(/^(?:[-*]\s*)?##\s+(.+)$/);
    if (heading) {
      flush();
      section = normalizeSection(heading[1]);
      inTable = false;
      continue;
    }

    const task = normalizedLine.match(TASK_RE);
    if (task) {
      flush();
      currentCard = cardFromTask(task, section);
      inTable = false;
      continue;
    }

    if (currentCard) {
      const property = line.match(PROPERTY_RE);
      if (property) {
        applyProperty(currentCard, property[1], property[2]);
        continue;
      }
      if (normalizedLine !== '' && !line.startsWith(' ') && !line.startsWith('\t')) {
        flush();
      }
    }

    if (section === 'other') {
      continue;
    }

    if (normalizedLine.startsWith('|')) {
      if (/^\|\s*-+/.test(normalizedLine) || /\|\s*Priority\s*\|/i.test(normalizedLine)) {
        inTable = true;
        continue;
      }
      if (inTable) {
        const card = parseIssueRow(normalizedLine, section);
        if (card) {
          cards.push(card);
        }
      }
    } else if (inTable && normalizedLine !== '') {
      inTable = false;
    }
  }

  flush();
  return dedupeCards(cards);
}

function normalizeSection(value: string): Section {
  const normalized = value.toLowerCase();
  if (normalized.includes('open issue')) {
    return 'open';
  }
  if (normalized.includes('closed issue') || normalized.includes('recently closed')) {
    return 'closed';
  }
  return 'other';
}

function cardFromTask(match: RegExpMatchArray, section: Section): ChainlinkCard {
  const marker = match[1].toUpperCase();
  const paddedId = match[2];
  const id = Number.parseInt(paddedId, 10);
  const title = cleanCell(match[3]);
  return {
    id,
    title: title || `Issue #${id}`,
    priority: 'medium',
    labels: [],
    time: '0m',
    status: marker === 'DONE' || section === 'closed' ? 'closed' : 'open',
    pageRef: `chainlink/issues/${paddedId}`
  };
}

function applyProperty(card: ChainlinkCard, key: string, value: string) {
  const normalized = key.toLowerCase();
  if (normalized === 'priority') {
    card.priority = cleanCell(value).toLowerCase();
  } else if (normalized === 'labels') {
    card.labels = parseLabels(value);
  } else if (normalized === 'time') {
    card.time = cleanCell(value);
  } else if (normalized === 'status') {
    const status = cleanCell(value).toLowerCase();
    if (status === 'closed') {
      card.status = 'closed';
    } else if (status === 'open') {
      card.status = 'open';
    }
  }
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
    priority: cleanCell(priority).toLowerCase(),
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

function dedupeCards(cards: ChainlinkCard[]): ChainlinkCard[] {
  const seen = new Set<string>();
  return cards.filter((card) => {
    const key = `${card.status}-${card.id}`;
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
}
