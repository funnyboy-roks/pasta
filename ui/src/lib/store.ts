import { onMount } from 'svelte';
import { writable } from 'svelte/store';

let last_set: number | null = null;

const hashStore = writable<string>('');
export const on_hash = <T>(fn: (hash: string) => T) => {
    onMount(() => {
        const hash_value = window.location.hash?.substring(1);
        hashStore.set(hash_value);
        fn(hash_value);
        const listener = () => {
            const hash_value = window.location.hash?.substring(1);
            hashStore.set(hash_value);
            if (!last_set || Date.now() - last_set > 10) {
                console.log('run', Date.now() - last_set!);
                fn(hash_value);
                last_set = null;
            }
        };
        window.addEventListener('hashchange', listener);
        return () => window.removeEventListener('hashchange', listener);
    });
};

export const set_hash = (hash: string) => {
    last_set = Date.now();
    window.location.hash = hash;
};

export const hash = {
    subscribe: hashStore.subscribe,
};
