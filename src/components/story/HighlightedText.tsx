import { useEffect, useRef } from "react";
import { cn } from "@/lib/utils";

interface Segment {
  text: string;
}

interface HighlightedTextProps {
  segments: Segment[] | null;
  activeIndex: number;
  fallbackContent: string;
}

export function HighlightedText({
  segments,
  activeIndex,
  fallbackContent,
}: HighlightedTextProps) {
  const activeRef = useRef<HTMLSpanElement>(null);

  // Auto-scroll active sentence into view
  useEffect(() => {
    if (activeRef.current) {
      activeRef.current.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  }, [activeIndex]);

  if (!segments || segments.length === 0) {
    return (
      <p className="whitespace-pre-wrap text-sm leading-relaxed text-card-foreground">
        {fallbackContent}
      </p>
    );
  }

  return (
    <p className="whitespace-pre-wrap text-sm leading-relaxed text-card-foreground">
      {segments.map((seg, i) => {
        const isActive = i === activeIndex;
        const isPast = activeIndex >= 0 && i < activeIndex;

        return (
          <span
            key={i}
            ref={isActive ? activeRef : undefined}
            className={cn(
              "transition-colors duration-200",
              isActive && "rounded bg-primary/20 text-primary",
              isPast && "text-muted-foreground",
            )}
          >
            {seg.text}
            {i < segments.length - 1 ? " " : ""}
          </span>
        );
      })}
    </p>
  );
}
