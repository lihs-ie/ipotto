/**
 * Candidate action kind available from an IPO list block.
 */
export type IpoListBlockActionKind =
  | "apply"
  | "application_detail"
  | "stock_detail"
  | "stock_link";

/**
 * Snapshot of the actionable state for an IPO list block.
 */
export interface IpoListBlockSnapshot {
  readonly index: number;
  readonly hasApplyButton: boolean;
  readonly hasApplicationDetailButton: boolean;
  readonly hasStockDetailButton: boolean;
  readonly hasStockLink: boolean;
}

/**
 * Returns the preferred action kind for a single IPO list block.
 */
export function resolveIpoListBlockActionKind(
  block: IpoListBlockSnapshot,
): IpoListBlockActionKind | null {
  if (block.hasApplyButton) {
    return "apply";
  }

  if (block.hasApplicationDetailButton) {
    return "application_detail";
  }

  if (block.hasStockDetailButton) {
    return "stock_detail";
  }

  if (block.hasStockLink) {
    return "stock_link";
  }

  return null;
}

/**
 * Returns the preferred block and action kind across duplicate company entries.
 */
export function selectPreferredIpoListBlockAction(
  blocks: readonly IpoListBlockSnapshot[],
): {
  readonly index: number;
  readonly kind: IpoListBlockActionKind;
} | null {
  let selected:
    | {
        readonly index: number;
        readonly kind: IpoListBlockActionKind;
        readonly priority: number;
      }
    | null = null;

  for (const block of blocks) {
    const kind = resolveIpoListBlockActionKind(block);
    if (kind === null) {
      continue;
    }

    const priority = getIpoListBlockActionPriority(kind);
    if (selected === null || priority > selected.priority) {
      selected = {
        index: block.index,
        kind,
        priority,
      };
    }
  }

  if (selected === null) {
    return null;
  }

  return {
    index: selected.index,
    kind: selected.kind,
  };
}

/**
 * Returns the action priority used for duplicate company entries.
 */
function getIpoListBlockActionPriority(kind: IpoListBlockActionKind): number {
  switch (kind) {
    case "apply":
      return 4;
    case "application_detail":
      return 3;
    case "stock_detail":
      return 2;
    case "stock_link":
      return 1;
  }
}
