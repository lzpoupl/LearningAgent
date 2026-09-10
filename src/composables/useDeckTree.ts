import { getSubdecks } from '../services/anki'
import type { Deck } from '../types/anki'

export interface DeckNode {
  deck: Deck
  children: DeckNode[]
  expanded?: boolean
}

export interface DeckRow {
  deck: Deck
  cardCount: number
  depth: number
  expanded?: boolean
  hasChildren: boolean
}

export async function loadDeckTree(): Promise<DeckNode[]> {
  const roots = await getSubdecks('')
  return Promise.all(roots.map(loadDeckNode))
}

async function loadDeckNode(deck: Deck): Promise<DeckNode> {
  const children = await getSubdecks(deck.path)

  return {
    deck,
    children: await Promise.all(children.map(loadDeckNode)),
    expanded: false,
  }
}

export function aggregateCardCount(node: DeckNode): number {
  return node.deck.cardCount + node.children.reduce(
    (total, child) => total + aggregateCardCount(child),
    0,
  )
}

export function flattenDecks(nodes: DeckNode[]): Deck[] {
  return nodes.flatMap(node => [
    {
      ...node.deck,
      cardCount: aggregateCardCount(node),
    },
    ...flattenDecks(node.children),
  ])
}

export function flattenVisibleDecks(nodes: DeckNode[], depth = 0): DeckRow[] {
  return nodes.flatMap(node => [
    {
      deck: node.deck,
      cardCount: aggregateCardCount(node),
      depth,
      expanded: node.expanded,
      hasChildren: node.children.length > 0,
    },
    ...(node.expanded ? flattenVisibleDecks(node.children, depth + 1) : []),
  ])
}

export function flattenDeckRows(nodes: DeckNode[], depth = 0): DeckRow[] {
  return nodes.flatMap(node => [
    {
      deck: node.deck,
      cardCount: aggregateCardCount(node),
      depth,
      expanded: node.expanded,
      hasChildren: node.children.length > 0,
    },
    ...flattenDeckRows(node.children, depth + 1),
  ])
}

export function findDeckNode(nodes: DeckNode[], deckPath: string): DeckNode | null {
  for (const node of nodes) {
    if (node.deck.path === deckPath) {
      return node
    }

    const found = findDeckNode(node.children, deckPath)
    if (found) {
      return found
    }
  }

  return null
}

export function getDescendantPaths(deckPath: string, nodes: DeckNode[]): string[] {
  const node = findDeckNode(nodes, deckPath)

  return node ? getNodePaths(node) : [deckPath]
}

function getNodePaths(node: DeckNode): string[] {
  return [node.deck.path, ...node.children.flatMap(getNodePaths)]
}
