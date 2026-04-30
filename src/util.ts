import type { PARSER_LANGUAGE } from "./types";

export function getPlaceholderText(language: PARSER_LANGUAGE): string {
	if (language === "typescript") {
		return ".test.tsx?$\n.spec.tsx?$\n.d.ts$";
	} else {
		return "vendor/\ngenerated/\n_test\\.go$";
	}
}
