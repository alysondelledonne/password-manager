// Plain-English meaning of the two star ratings. The thresholds live in Rust
// (crates/pm-core/src/passphrase/rating.rs).

export interface GuessRating {
  stars: number;
  title: string;
  /** Finishes "fine to memorize for …". */
  goodFor: string;
  /** Follows "goodFor:" in the guide. */
  examples: string;
}

export interface MemoryRating {
  stars: number;
  title: string;
  description: string;
}

export const GUESS_RATINGS: GuessRating[] = [
  {
    stars: 5,
    title: "Excellent",
    goodFor: "accounts that unlock your other accounts",
    examples: "your main email, your Apple, Google or Microsoft account.",
  },
  {
    stars: 4,
    title: "Strong",
    goodFor: "accounts with money or private details",
    examples: "online banking, shopping sites that keep your card, health or work accounts.",
  },
  {
    stars: 3,
    title: "Good",
    goodFor: "everyday accounts",
    examples: "social media, streaming services, shopping without a saved card.",
  },
  {
    stars: 2,
    title: "Fair",
    goodFor: "low-stakes accounts",
    examples: "news sites, forums, games without purchases.",
  },
  {
    stars: 1,
    title: "Basic",
    goodFor: "throwaway sign-ups",
    examples: "newsletters, free trials, guest Wi-Fi.",
  },
];

export const MEMORY_RATINGS: MemoryRating[] = [
  { stars: 5, title: "Very easy", description: "Sticks after using it once or twice." },
  { stars: 4, title: "Easy", description: "Sticks after a few uses." },
  { stars: 3, title: "Some practice", description: "Type it a few times today and it will stick." },
  { stars: 2, title: "Hard", description: "Hard to keep in your head without writing it down." },
  { stars: 1, title: "Very hard", description: "Not realistic to memorize." },
];

export function guessRating(stars: number): GuessRating {
  return GUESS_RATINGS.find((r) => r.stars === stars) ?? GUESS_RATINGS[GUESS_RATINGS.length - 1];
}

/** One sentence that weighs both ratings. */
export function verdict(memoryStars: number, guessStars: number): string {
  const goodFor = guessRating(guessStars).goodFor;
  if (memoryStars <= 2) {
    return "Hard to remember. Fewer digits or special characters, or your own words, would make it easier.";
  }
  if (guessStars >= 3) {
    return `Good balance: fine to memorize for ${goodFor}.`;
  }
  return `Easy to remember, but only strong enough for ${goodFor}. Random words, or more of your own words, make it harder to guess.`;
}
