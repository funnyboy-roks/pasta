<script lang="ts">
    import * as Dialog from '$lib/components/ui/dialog';
    import { Input } from '$lib/components/ui/input';
    import { Button } from '$lib/components/ui/button';

    import Link from '@lucide/svelte/icons/link';
    import EyeIcon from '@lucide/svelte/icons/eye';
    import EyeOffIcon from '@lucide/svelte/icons/eye-off';
    import { save_content } from '$lib/save';
    import { toast } from 'svelte-sonner';

    let url = $state('');

    let show_password = $state(false);
    let password = $state('');
    let error = $state('');

    let saving = $state(false);

    let valid = $derived.by(() => {
        return URL.parse(url) !== null
    });

    const { open = $bindable(false) } = $props();

    const save = () => {
        const save_inner = async () => {
            saving = true;

            const res = await save_content(url, password, 'application/link');

            saving = false;

            switch (res.status) {
                case 'ok': {
                    const link = `${location.origin}/#${res.slug}`;
                    return link;
                }
                case 'error': {
                    throw res.message;
                }
            }
        };

        toast.promise(save_inner, {
            loading: 'Saving...',
            success: (link) => `Saved at ${link}\nCopied to clipboard`,
            error: (reason) => `Error saving pasta:\n${reason}`,
        });
    };

    const submit = (e: SubmitEvent) => {
        e.preventDefault();
        if (!valid) return;
        save();
    };
</script>

<Dialog.Root open={open}>
    <Dialog.Content>
        <Dialog.Header>
            <div class="my-5 flex w-full flex-row justify-center">
                <Link size={96} />
            </div>
            <Dialog.Title>Create a new link</Dialog.Title>
            <Dialog.Description>
                Create a pasta that will redirect the user to another url
            </Dialog.Description>
        </Dialog.Header>
        <form onsubmit={submit}>
            <div class="flex flex-col gap-1">
                <div class="flex flex-col gap-1">
                    <Input
                        type="url"
                        autofocus
                        bind:value={url}
                        placeholder="URL"
                    />
                    <div class="flex flex-row gap-1">
                        <Input
                            type={show_password ? 'text' : 'password'}
                            bind:value={password}
                            placeholder="Password (Optional)"
                        />
                        <Button
                            variant="outline"
                            size="icon"
                            onclick={() => (show_password = !show_password)}
                        >
                            {#if show_password}
                                <EyeIcon />
                            {:else}
                                <EyeOffIcon />
                            {/if}
                        </Button>
                    </div>
                    {#if error}
                        <div class="text-destructive">
                            {error}
                        </div>
                    {/if}
                </div>
                <div class="flex flex-row justify-end">
                    <Button type="submit" class="w-1/3 self-end" disabled={saving || !valid}>
                        Create Link
                    </Button>
                </div>
            </div>
        </form>
    </Dialog.Content>
</Dialog.Root>
