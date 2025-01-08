import {LRUCache} from "lru-cache";
import {ICache, ICacheFactory} from "./cache";

export const Cache: ICacheFactory = {
    create: ({ ttlSeconds } = {}) => {
        return new InMemoryCache(ttlSeconds);
    }
}

export class InMemoryCache<T extends {} = any> implements ICache<T> {
    private readonly cache: LRUCache<string, T>;

    constructor(ttlSeconds = 600) {
        this.cache = new LRUCache({
            max: 1000,
            ttl: Math.max(ttlSeconds, 1) * 1000,
            ttlAutopurge: false,
            allowStale: false,
        });
    }

    get(key: string): T | undefined {
        return this.cache.get(key);
    }

    set(key: string, value: T, ttlSeconds?: number): T {
        const ttl = ttlSeconds ? Math.max(ttlSeconds, 1) * 1000 : this.cache.ttl;
        this.cache.set(key, value, { ttl });
        return value;
    }

    has(key: string): boolean {
        return this.cache.has(key);
    }

    delete(key: string): void {
        this.cache.delete(key);
    }

    clear(): void {
        this.cache.clear();
    }
}
