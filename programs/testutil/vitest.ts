import { expect } from 'vitest';

/**
 * Expects a snapshot-friendly object with volatile fields masked.
 * Key-path fields and volatile string values are replaced with the matched pattern string.
 */
export function expectMasked(
  actual: any | Promise<any>,
  options?: {
    keys?: (string | RegExp)[];
    values?: (string | RegExp)[];
  },
  message?: string
) {
  return expect(masked(actual, options), message);
}

/**
 * Recursively masks volatile fields in an object. Also transform original object by toJSON if the given object has the method.
 * Keys matched by `options.keys` (by key or dot-path) are replaced with the matched pattern string.
 * String values matched by `options.values` are also replaced with the matched pattern string.
 */
export async function masked(
  actual: any | Promise<any>,
  options?: {
    extraKeys?: (string | RegExp)[];
    extraValues?: (string | RegExp)[];
    keys?: (string | RegExp)[]; // overriding all
    values?: (string | RegExp)[]; // overriding all
    maxDepth?: number;
  }
) {
  const keys = options?.keys ?? [
    /[.*S|s]lots?$/,
    /.*At?$/,
    'signature',
    /[.*C|c]ontribution?$/,
    ...(options?.extraKeys ?? []),
  ];
  const values = options?.values ?? [
    /^Program \w{32,44} consumed \d+ of \d+ compute units$/,
    ...(options?.extraValues ?? []),
  ];
  actual = await actual;
  if (typeof actual.toJSON == 'function') {
    actual = actual.toJSON();
  }

  const maxDepth = options?.maxDepth ?? 100;

  function walk(value: any, path: string[] = []): any {
    if (path.length >= maxDepth) return value;

    if (Array.isArray(value)) {
      return value.map((item, i) => walk(item, [...path, String(i)]));
    }

    if (typeof value === 'object' && value !== null) {
      const result: any = {};
      for (const [key, val] of Object.entries(value)) {
        const fullPath = [...path, key].join('.');

        const masked = keys.find((pattern) =>
          typeof pattern === 'string'
            ? fullPath === pattern || key === pattern
            : pattern.test(fullPath)
        );

        if (masked) {
          result[key] = `MASKED(${masked.toString()})`;
        } else {
          result[key] = walk(val, [...path, key]);
        }
      }
      return result;
    }

    if (typeof value === 'string') {
      const masked = values.find((pattern) =>
        typeof pattern === 'string'
          ? value.includes(pattern)
          : pattern.test(value)
      );
      if (masked) {
        return `MASKED(${masked.toString()})`;
      }
      return value;
    }

    return value;
  }

  return walk(actual);
}
