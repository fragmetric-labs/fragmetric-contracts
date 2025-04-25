import chalk from 'chalk';

export type ContextDescription = {
  label: string;
  mutable: boolean; // is a mutable context like transaction template?
  unresolved: boolean; // is a unresolved context?
  unused: boolean; // is the context not being used? like empty account
  properties: Record<string, any>; // arbitrary properties
};

export type ContextTree = ContextNode & {
  children: ContextTree[];
  skippedEdges: [string, Context<any>][];
};

export type ContextNode = {
  context: Context<any>;
  parent: Context<any> | null;
  path: string[];
  depth: number;
};

export type ContextNodeVisitor = (node: ContextNode) => {
  in: number; // remaining inbound level to search
  out: number; // remaining outbound level to search
};

export function setContextCustomInspectionEnabled(enabled: boolean) {
  Context.__customInspectionEnabled = enabled;
}

export abstract class Context<P extends Context<any> | null> {
  abstract readonly parent: P;
  protected readonly symbol: symbol = Symbol.for(this.constructor.name);

  toJSON(): any {
    return this.toContextDescription();
  }

  protected __findParentContext<T extends Context<any>>(
    constructor: new (...args: any[]) => T
  ): T | null {
    const symbol = Symbol.for(constructor.name);
    let current: Context<any> | null = this.parent;
    while (current) {
      if (current.symbol === symbol || current instanceof constructor) {
        break;
      }
      current = current.parent;
    }
    return current as T | null;
  }

  private readonly __memoizedMap = new Map<string, any>();
  protected __memoized<T>(key: string, calculator: () => T) {
    if (this.__memoizedMap.has(key)) {
      return this.__memoizedMap.get(key) as T;
    }
    const value = calculator();
    this.__memoizedMap.set(key, value);
    return value;
  }

  static readonly toJSONReplacer = (_: string, value: any) =>
    typeof value === 'bigint' ? value.toString() : value;
  private readonly __deduplicatedMap = new Map<string, Promise<any>>();
  protected async __deduplicated<T>(
    config: {
      method: string;
      params: any[];
      alternativeParams?: any[] | null;
      intervalSeconds?: number; // default is 5s
    },
    resolver: () => Promise<T>
  ): Promise<T> {
    const key = `${config.method}:${JSON.stringify(config.params, Context.toJSONReplacer)}`;
    let existingResolver = this.__deduplicatedMap.get(key);
    if (!existingResolver && config.alternativeParams) {
      const alternativeKey = `${config.method}:${JSON.stringify(config.alternativeParams, Context.toJSONReplacer)}`;
      existingResolver = this.__deduplicatedMap.get(alternativeKey);
    }
    if (existingResolver) {
      return existingResolver as Promise<T>;
    }
    const newResolver = resolver().finally(() => {
      setTimeout(
        () => this.__deduplicatedMap.delete(key),
        (isNaN(config.intervalSeconds as number)
          ? 5
          : config.intervalSeconds!) * 1000
      );
    });
    this.__deduplicatedMap.set(key, newResolver);
    return newResolver;
  }

  protected __getChildContextEntries(): [string, Context<any>][] {
    const entries: [string, Context<any>][] = [];
    for (const [key, value] of Object.entries(this)) {
      const items = Array.isArray(value) ? value : [value];
      for (let i = 0; i < items.length; i++) {
        const v = items[i];
        if (
          v instanceof Context &&
          v !== this.parent &&
          !key.startsWith('__')
        ) {
          entries.push([Array.isArray(value) ? `${key}.${i}` : key, v]);
        }
      }
    }
    return entries;
  }

  protected __visitContextGraph(
    visitor: ContextNodeVisitor,
    visited = new Set<Context<any>>(),
    skipped = new Map<Context<any>, [string, Context<any>][]>(),
    depth = 0,
    path: string[] = [],
    parent: Context<any> | null = null
  ): void {
    if (visited.has(this)) return;
    visited.add(this);

    const { in: remainingIn, out: remainingOut } = visitor({
      context: this,
      parent,
      path,
      depth,
    });

    // traverse out-edges (children)
    if (depth >= 0) {
      for (const [key, child] of this.__getChildContextEntries()) {
        if (remainingOut > 0) {
          child.__visitContextGraph(
            visitor,
            visited,
            skipped,
            depth + 1,
            path.concat(key),
            this
          );
        } else {
          let skippedEdges = skipped.get(this);
          if (!skippedEdges) {
            skipped.set(this, (skippedEdges = []));
          }
          if (skippedEdges.every((e) => e[1] != child)) {
            skippedEdges.push([key, child]);
          }
        }
      }
    }

    // traverse in-edge (.parent)
    if (this.parent && depth <= 0) {
      if (remainingIn > 0) {
        this.parent.__visitContextGraph(
          visitor,
          visited,
          skipped,
          depth - 1,
          path.concat('parent'),
          this
        );
      } else {
        // let skippedEdges = skipped.get(this);
        // if (!skippedEdges) {
        //   skipped.set(this, skippedEdges = []);
        // }
        // skippedEdges.push(['parent', this.parent]);
      }
    }
  }

  protected __buildContextTree(visitor: ContextNodeVisitor): ContextTree {
    const treeMap = new Map<Context<any>, ContextTree>();
    const visited = new Set<Context<any>>();
    const skipped = new Map<Context<any>, [string, Context<any>][]>();

    this.__visitContextGraph(
      (node) => {
        const res = visitor(node);

        treeMap.set(node.context, {
          ...node,
          children: [],
          skippedEdges: [],
        });

        return res;
      },
      visited,
      skipped
    );

    for (const tree of treeMap.values()) {
      if (tree.parent) {
        const parentTree = treeMap.get(tree.parent);
        if (parentTree) {
          parentTree.children.push(tree);
        }
      }
      tree.skippedEdges = skipped.get(tree.context) ?? tree.skippedEdges;
    }

    let root = treeMap.get(this)!;
    while (root.parent && treeMap.has(root.parent)) {
      root = treeMap.get(root.parent)!;
    }
    return root;
  }

  public toContextDescription(): ContextDescription {
    return {
      label: (this.symbol.description ?? this.symbol.toString()).replace(
        /(Context)$/,
        ''
      ),
      // .replace(/(Account|Template)$/, ''),
      mutable: false,
      unresolved: false,
      unused: false,
      properties: {},
    };
  }

  public toContextTreeString(options?: {
    maxOut?: number;
    maxIn?: number;
    maxLineWidth?: number;
    colorized?: boolean;
    multiline?: boolean;
  }): string {
    const {
      maxOut = 5,
      maxIn = 1,
      maxLineWidth = 200,
      colorized = false,
      multiline = false,
    } = options ?? {};

    const tree = this.__buildContextTree((node) => ({
      out: Math.min(maxOut, maxOut - node.depth),
      in: Math.min(maxIn, maxIn + node.depth),
    }));

    const entries: [string, string, ContextDescription][] = []; // prefix, key, value
    const collectEntries = (
      node: ContextTree,
      prevPrefix: string = '',
      isLast: boolean = true
    ) => {
      const key =
        node.path.length === 0 ? '(this)' : node.path[node.path.length - 1];
      const prefix = `${prevPrefix}${node.path.length === 0 ? '' : isLast ? '└── ' : '├── '}`;
      const desc = node.context.toContextDescription();
      entries.push([prefix, key, desc]);

      const nextPrefix =
        prevPrefix + (node.path.length === 0 ? '' : isLast ? '    ' : '│   ');
      const lastIndex = node.children.length - 1;
      node.children.forEach((child, i) => {
        collectEntries(child, nextPrefix, i === lastIndex);
      });

      if (node.skippedEdges.length > 0) {
        entries.push([
          `${nextPrefix}└── `,
          `+${node.skippedEdges.length} more`,
          {
            label: node.skippedEdges.map(([k]) => k).join(', '),
            mutable: false,
            unresolved: true,
            unused: false,
            properties: {},
          },
        ]);
      }
    };

    collectEntries(tree);

    const maxPrefixedKeyLength =
      Math.max(...entries.map(([prefix, key]) => prefix.length + key.length)) +
      4;
    return entries
      .map(([prefix, key, desc], index) => {
        const paddedKey = key.padEnd(maxPrefixedKeyLength - prefix.length, ' ');
        let left = prefix;
        if (colorized) {
          if (desc.mutable) {
            left += chalk.yellow(paddedKey);
          } else if (desc.unused) {
            left += chalk.italic.dim(paddedKey);
          } else if (desc.unresolved) {
            left += chalk.dim(paddedKey);
          } else {
            left += paddedKey;
          }
        } else {
          left += paddedKey;
        }
        const rights = Context.formatContextDescription(
          desc,
          colorized,
          Math.max(maxLineWidth - maxPrefixedKeyLength, 1)
        );

        const lines = rights.map((right, i) => {
          if (i == 0) {
            return `${left}${right}`;
          } else {
            const nextPrefix = entries[index + 1]?.[0] ?? '';
            return `${nextPrefix.replace(/(├── |└── )$/, '│   ').padEnd(maxPrefixedKeyLength, ' ')}${right}`;
          }
        });
        return multiline || index == 0 ? lines.join('\n') : lines[0];
      })
      .join('\n');
  }

  private static formatContextDescription(
    desc: ContextDescription,
    colorized = false,
    maxLineWidth = 200
  ): string[] {
    const labelWidth = desc.label.length + 1;
    let props = Object.entries(desc.properties)
      .map(([k, v]) => `${k}=${v}`)
      .join(', ');
    const lines: string[] = [];
    while (props.length) {
      const maxPropsWidth =
        lines.length == 0 ? maxLineWidth - labelWidth : maxLineWidth;
      if (props.length > maxPropsWidth) {
        lines.push(props.substring(0, maxPropsWidth));
        props = props.substring(maxPropsWidth);
      } else {
        lines.push(props.substring(0, maxPropsWidth));
        props = '';
      }
    }
    if (!lines.length) {
      lines.push('');
    }
    lines.forEach((line, i) => {
      if (i == 0) {
        lines[i] =
          `${colorized ? chalk.blue(desc.label) : desc.label} ${colorized ? chalk.dim(lines[i]) : lines[i]}`;
      } else {
        lines[i] = colorized ? chalk.dim(lines[i]) : lines[i];
      }
    });
    return lines;
  }

  toString(): string {
    return Context.formatContextDescription(
      this.toContextDescription(),
      false,
      1000
    ).join('');
  }

  static __customInspectionEnabled = false;

  [Symbol.for('nodejs.util.inspect.custom')](
    depth: number,
    inspectOptions: any,
    inspect: any
  ) {
    if (Context.__customInspectionEnabled) {
      const colorized = inspectOptions?.colors == true;
      const indent =
        ((isNaN(inspectOptions.depth) ? depth : inspectOptions.depth) - depth) *
        2;
      const maxLineWidth = (process.stdout.columns ?? 200) - indent;
      if (inspectOptions?.compact === true || inspectOptions?.depth === 1) {
        const line = Context.formatContextDescription(
          this.toContextDescription(),
          colorized,
          1000
        ).join('');
        return line.length > maxLineWidth
          ? line.substring(0, maxLineWidth - 3) + '...'
          : line;
      }
      return this.toContextTreeString({ maxLineWidth, colorized });
    }
    return this;
  }

  /**
   Intended to asynchronously fetch, compute, or aggregate any custom,
   human-readable metadata relevant to this context node.
  */
  abstract resolve(noCache?: boolean): Promise<any>;
}
