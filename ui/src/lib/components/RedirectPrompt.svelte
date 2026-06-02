<script lang="ts">
    import * as Dialog from '$lib/components/ui/dialog';
    import { Button } from '$lib/components/ui/button';

    import Link from '@lucide/svelte/icons/link';

    const { redirect = '' } = $props();
</script>

<Dialog.Root open={!!redirect}>
    <Dialog.Content
        showCloseButton={false}
        escapeKeydownBehavior="defer-otherwise-ignore"
        interactOutsideBehavior="defer-otherwise-ignore"
        trapFocus
        preventScroll
    >
        <Dialog.Header>
            <div class="my-5 flex w-full flex-row justify-center">
                <Link size={96} />
            </div>
            <Dialog.Title>This Pasta would like redirect you to the following URL</Dialog.Title>
            <Dialog.Description class="text-xl">
                {redirect}
            </Dialog.Description>
        </Dialog.Header>
        <div class="flex flex-col gap-1">
            <div class="flex flex-row justify-between">
                <Button href={redirect} class="w-full">Proceed</Button>
            </div>
        </div>
    </Dialog.Content>
</Dialog.Root>
