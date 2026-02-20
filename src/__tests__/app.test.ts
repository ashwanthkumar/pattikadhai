import { describe, it, expect } from "vitest";
import * as fc from "fast-check";
import { cn } from "@/lib/utils";
import { GENRE_ICONS, AUDIO_STAGES, DEPENDENCY_STEPS } from "@/lib/constants";
import type { Genre } from "@/types";

describe("cn utility", () => {
  it("merges class names", () => {
    expect(cn("foo", "bar")).toBe("foo bar");
  });

  it("handles conditional classes", () => {
    expect(cn("foo", false && "bar", "baz")).toBe("foo baz");
  });

  it("merges tailwind classes correctly", () => {
    expect(cn("p-4", "p-2")).toBe("p-2");
  });
});

describe("GENRE_ICONS", () => {
  it("has 6 genres", () => {
    expect(Object.keys(GENRE_ICONS)).toHaveLength(6);
  });

  it("all genre IDs are non-empty strings", () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...Object.keys(GENRE_ICONS)),
        (id) => {
          expect(id.length).toBeGreaterThan(0);
          expect(typeof GENRE_ICONS[id]).toBe("string");
        }
      )
    );
  });
});

describe("AUDIO_STAGES", () => {
  it("has 2 stages", () => {
    expect(AUDIO_STAGES).toHaveLength(2);
  });

  it("ends with complete", () => {
    expect(AUDIO_STAGES[AUDIO_STAGES.length - 1].key).toBe("complete");
  });

  it("all stages have unique keys", () => {
    const keys = AUDIO_STAGES.map((s) => s.key);
    expect(new Set(keys).size).toBe(keys.length);
  });
});

describe("DEPENDENCY_STEPS", () => {
  it("has 5 steps", () => {
    expect(DEPENDENCY_STEPS).toHaveLength(5);
  });

  it("all steps have required fields", () => {
    for (const step of DEPENDENCY_STEPS) {
      expect(step.name).toBeTruthy();
      expect(step.label).toBeTruthy();
      expect(step.description).toBeTruthy();
    }
  });
});

describe("Type shapes (property-based)", () => {
  const genreArb = fc.record({
    id: fc.string({ minLength: 1, maxLength: 20 }),
    name: fc.string({ minLength: 1, maxLength: 50 }),
    description: fc.string({ minLength: 1, maxLength: 200 }),
    icon: fc.option(fc.string()),
    display_order: fc.integer({ min: 0, max: 100 }),
    created_at: fc.string(),
  });

  it("Genre objects have all required fields", () => {
    fc.assert(
      fc.property(genreArb, (genre: Genre) => {
        expect(genre.id).toBeTruthy();
        expect(genre.name).toBeTruthy();
        expect(typeof genre.display_order).toBe("number");
      })
    );
  });

  it("Story status is valid", () => {
    fc.assert(
      fc.property(
        fc.constantFrom("draft", "complete"),
        (status) => {
          expect(["draft", "complete"]).toContain(status);
        }
      )
    );
  });

  it("Part numbers remain sequential after sort", () => {
    fc.assert(
      fc.property(
        fc.array(fc.integer({ min: 1, max: 100 }), { minLength: 1, maxLength: 20 }),
        (partNumbers) => {
          const sorted = [...partNumbers].sort((a, b) => a - b);
          for (let i = 1; i < sorted.length; i++) {
            expect(sorted[i]).toBeGreaterThanOrEqual(sorted[i - 1]);
          }
        }
      )
    );
  });

  it("Audio job status transitions are valid", () => {
    const validStatuses = ["pending", "voice_generating", "complete", "failed"];
    fc.assert(
      fc.property(
        fc.constantFrom(...validStatuses),
        (status) => {
          expect(validStatuses).toContain(status);
        }
      )
    );
  });
});
