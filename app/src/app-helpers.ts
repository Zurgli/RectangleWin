export const SECTION_ACTIONS = [
  "FirstThird",
  "FirstTwoThirds",
  "CenterThird",
  "LastTwoThirds",
  "LastThird",
] as const;

export const SECTIONS: { title: string; actions: readonly string[] }[] = [
  { title: "Halves", actions: ["LeftHalf", "RightHalf", "TopHalf", "BottomHalf"] },
  { title: "Quarters", actions: ["UpperLeft", "UpperRight", "LowerLeft", "LowerRight"] },
  { title: "Thirds", actions: SECTION_ACTIONS },
  { title: "Other", actions: ["Maximize", "Center", "Undo", "NextDisplay", "PreviousDisplay"] },
];

export type SectionLayoutKind = "Thirds" | "Fourths" | "Fifths";

export type ShortcutKeypress = {
  key: string;
  metaKey: boolean;
  altKey: boolean;
  ctrlKey: boolean;
  shiftKey: boolean;
};

export type ShortcutTileGridSpec = {
  templateColumns: string;
  templateRows: string;
  col: number;
  row: number;
  colSpan?: number;
  rowSpan?: number;
};

export function normalizeSectionLayoutKind(value: string | null | undefined): SectionLayoutKind {
  if (value === "Fifths") {
    return "Fifths";
  }
  if (value === "Fourths") {
    return "Fourths";
  }
  return "Thirds";
}

export function thirdsSectionActionLabel(action: string): string {
  const labels: Record<string, string> = {
    FirstThird: "Left",
    FirstTwoThirds: "Left two",
    CenterThird: "Center",
    LastTwoThirds: "Right two",
    LastThird: "Right",
  };
  return labels[action] ?? action;
}

export function getSectionLayoutLinePositions(kind: SectionLayoutKind, width: number): number[] {
  if (kind === "Thirds") {
    return [width / 3, (2 * width) / 3];
  }
  if (kind === "Fourths") {
    return [width / 4, (3 * width) / 4];
  }
  return [width / 5, (4 * width) / 5];
}

export function keyToShortcutLabel(key: string): string {
  const map: Record<string, string> = {
    ArrowLeft: "Left",
    ArrowRight: "Right",
    ArrowUp: "Up",
    ArrowDown: "Down",
    Enter: "Enter",
    Delete: "Delete",
  };
  return map[key] ?? key;
}

export function getShortcutFromKeypress(keypress: ShortcutKeypress): string {
  const parts: string[] = [];
  if (keypress.metaKey) parts.push("Win");
  if (keypress.altKey) parts.push("Alt");
  if (keypress.ctrlKey) parts.push("Ctrl");
  if (keypress.shiftKey) parts.push("Shift");
  parts.push(keyToShortcutLabel(keypress.key));
  return parts.join("+");
}

export function getShortcutTileGridSpec(
  action: string,
  sectionLayout: SectionLayoutKind = "Thirds"
): ShortcutTileGridSpec | undefined {
  const sectionCols =
    sectionLayout === "Fifths"
      ? "1fr 3fr 1fr"
      : sectionLayout === "Fourths"
        ? "1fr 2fr 1fr"
        : "1fr 1fr 1fr";

  const specs: Record<string, ShortcutTileGridSpec> = {
    LeftHalf: { templateColumns: "1fr 1fr", templateRows: "1fr", col: 1, row: 1 },
    RightHalf: { templateColumns: "1fr 1fr", templateRows: "1fr", col: 2, row: 1 },
    TopHalf: { templateColumns: "1fr", templateRows: "1fr 1fr", col: 1, row: 1 },
    BottomHalf: { templateColumns: "1fr", templateRows: "1fr 1fr", col: 1, row: 2 },
    UpperLeft: { templateColumns: "1fr 1fr", templateRows: "1fr 1fr", col: 1, row: 1 },
    UpperRight: { templateColumns: "1fr 1fr", templateRows: "1fr 1fr", col: 2, row: 1 },
    LowerLeft: { templateColumns: "1fr 1fr", templateRows: "1fr 1fr", col: 1, row: 2 },
    LowerRight: { templateColumns: "1fr 1fr", templateRows: "1fr 1fr", col: 2, row: 2 },
    FirstThird: { templateColumns: sectionCols, templateRows: "1fr", col: 1, row: 1, colSpan: 1 },
    FirstTwoThirds: {
      templateColumns: sectionCols,
      templateRows: "1fr",
      col: 1,
      row: 1,
      colSpan: 2,
    },
    CenterThird: { templateColumns: sectionCols, templateRows: "1fr", col: 2, row: 1, colSpan: 1 },
    LastTwoThirds: {
      templateColumns: sectionCols,
      templateRows: "1fr",
      col: 2,
      row: 1,
      colSpan: 2,
    },
    LastThird: { templateColumns: sectionCols, templateRows: "1fr", col: 3, row: 1, colSpan: 1 },
    Maximize: { templateColumns: "1fr", templateRows: "1fr", col: 1, row: 1 },
    Center: {
      templateColumns: "1fr 1fr 1fr",
      templateRows: "1fr 1fr 1fr",
      col: 2,
      row: 2,
      colSpan: 1,
      rowSpan: 1,
    },
    NextDisplay: { templateColumns: "1fr 1fr", templateRows: "1fr", col: 2, row: 1 },
    PreviousDisplay: { templateColumns: "1fr 1fr", templateRows: "1fr", col: 1, row: 1 },
  };

  return specs[action];
}
