<script lang="ts">
	import './layout.css';
    import { ModeWatcher } from 'mode-watcher';
	import favicon from '$lib/assets/favicon.svg';
	import { Toaster } from '$lib/components/ui/sonner';
	import { PUBLIC_PASTA_UI_API } from '$env/static/public';

	let { children } = $props();
</script>

<svelte:head>
    <link rel="icon" href={favicon} />
</svelte:head>

<noscript>
    <div class="fixed z-100 w-screen h-screen bg-slate-800 text-white text-center flex flex-col justify-center items-center gap-5">
        <div class="text-3xl">
            Unfortunately, this application requires JavaScript :(
        </div>
        <div class="text-xl">
            Fear not!  You can still upload a pasta with the following command:
        </div>
        <div class="min-w-1/4 text-start">
            <div class="w-full rounded-xl bg-gray-900 px-4 py-2 font-mono text-white">
                { `$ curl -T <file> ${PUBLIC_PASTA_UI_API}/post` }
            </div>
        </div>
        <div class="text-xl">
            Which will print a slug that you can then use in this command to fetch the data:
        </div>
        <div class="min-w-1/4 text-start">
            <div class="w-full rounded-xl bg-gray-900 px-4 py-2 font-mono text-white">
                { `$ curl ${PUBLIC_PASTA_UI_API}/<slug>` }
            </div>
        </div>
    </div>
</noscript>

<ModeWatcher />
<Toaster position="top-center" richColors={true} />
{@render children?.()}
