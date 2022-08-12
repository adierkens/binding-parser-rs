import { VALID_AST_PARSER_TESTS } from "./ast-cases.mjs";
import {
  parseCustom,
  parseEBNF,
  parseParsimmon,
} from "@player-ui/binding-grammar";
import { parse as parseRS } from "../../pkg/binding_parser_rs.js";

const parsers = [
  { name: "parsimmon", parser: parseParsimmon },
  { name: "ebnf", parser: parseEBNF },
  { name: "custom", parser: parseCustom },
  { name: "rust", parser: parseRS },
];

const VALID_ITERATIONS = 1000;

/** Execute 1 iteration of the parser test */
const runOnce = (parser) => {
  const start = Date.now();

  for (const testCase of VALID_AST_PARSER_TESTS) {
    parser(testCase[0]);
  }

  const end = Date.now();

  return end - start;
};

/** Run all parser perf tests */
export const testAll = () => {
  const results = [];

  for (const parser of parsers) {
    const runs = [];

    for (let i = 0; i < VALID_ITERATIONS; i++) {
      runs.push(runOnce(parser.parser));
    }

    const total = runs.reduce((s, n) => s + n);
    const opsPerSec =
      (VALID_ITERATIONS * VALID_AST_PARSER_TESTS.length) / (total / 1000);
    results.push({
      name: parser.name,
      time: total,
      opsPerSec,
    });
  }

  console.table(results);
};

testAll();
