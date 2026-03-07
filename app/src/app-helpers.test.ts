import { describe, expect, it } from "vitest";

import {
  SECTION_ACTIONS,
  SECTIONS,
  getSectionLayoutLinePositions,
  getShortcutFromKeypress,
  getShortcutTileGridSpec,
  keyToShortcutLabel,
  normalizeSectionLayoutKind,
  thirdsSectionActionLabel,
} from "./app-helpers";

describe("app helpers", () => {
  it("defines the section groupings used by the UI", () => {
    expect(SECTION_ACTIONS).toEqual([
      "FirstThird",
      "FirstTwoThirds",
      "CenterThird",
      "LastTwoThirds",
      "LastThird",
    ]);
    expect(SECTIONS.map((section) => section.title)).toEqual([
      "Halves",
      "Quarters",
      "Thirds",
      "Other",
    ]);
    expect(SECTIONS.find((section) => section.title === "Thirds")?.actions).toEqual(SECTION_ACTIONS);
  });

  it("normalizes section layout kinds", () => {
    expect(normalizeSectionLayoutKind("Thirds")).toBe("Thirds");
    expect(normalizeSectionLayoutKind("Fourths")).toBe("Fourths");
    expect(normalizeSectionLayoutKind("Fifths")).toBe("Fifths");
    expect(normalizeSectionLayoutKind("unexpected")).toBe("Thirds");
    expect(normalizeSectionLayoutKind(undefined)).toBe("Thirds");
  });

  it("maps thirds labels for display", () => {
    expect(thirdsSectionActionLabel("FirstThird")).toBe("Left");
    expect(thirdsSectionActionLabel("CenterThird")).toBe("Center");
    expect(thirdsSectionActionLabel("LastThird")).toBe("Right");
    expect(thirdsSectionActionLabel("CustomAction")).toBe("CustomAction");
  });

  it("maps keyboard event keys to shortcut labels", () => {
    expect(keyToShortcutLabel("ArrowLeft")).toBe("Left");
    expect(keyToShortcutLabel("ArrowDown")).toBe("Down");
    expect(keyToShortcutLabel("Enter")).toBe("Enter");
    expect(keyToShortcutLabel("A")).toBe("A");
  });

  it("builds shortcut labels in the expected modifier order", () => {
    expect(
      getShortcutFromKeypress({
        key: "ArrowLeft",
        metaKey: true,
        altKey: true,
        ctrlKey: false,
        shiftKey: true,
      })
    ).toBe("Win+Alt+Shift+Left");

    expect(
      getShortcutFromKeypress({
        key: "Delete",
        metaKey: false,
        altKey: false,
        ctrlKey: true,
        shiftKey: false,
      })
    ).toBe("Ctrl+Delete");
  });

  it("returns the expected section layout divider positions", () => {
    expect(getSectionLayoutLinePositions("Thirds", 18)).toEqual([6, 12]);
    expect(getSectionLayoutLinePositions("Fourths", 20)).toEqual([5, 15]);
    expect(getSectionLayoutLinePositions("Fifths", 25)).toEqual([5, 20]);
  });

  it("returns grid specs for shortcut tile rendering", () => {
    expect(getShortcutTileGridSpec("LeftHalf")).toEqual({
      templateColumns: "1fr 1fr",
      templateRows: "1fr",
      col: 1,
      row: 1,
    });

    expect(getShortcutTileGridSpec("CenterThird", "Fifths")).toEqual({
      templateColumns: "1fr 3fr 1fr",
      templateRows: "1fr",
      col: 2,
      row: 1,
      colSpan: 1,
    });

    expect(getShortcutTileGridSpec("UnknownAction")).toBeUndefined();
  });
});
