<script lang="ts">
    import * as Dialog from '$lib/components/ui/dialog';
    import { Button, buttonVariants } from '$lib/components/ui/button';
    import Link from '@lucide/svelte/icons/link';
    import Git from '@lucide/svelte/icons/git-fork';

    let { open = $bindable(false) } = $props();
</script>

<Dialog.Root bind:open={open}>
    <Dialog.Content class="sm:max-w-4xl">
        <Dialog.Header>
            <Dialog.Title>About Pasta</Dialog.Title>
            <Dialog.Description>
                What is a Pasta?
            </Dialog.Description>
        </Dialog.Header>
        <div class="flex flex-col gap-2">
            <h1 class="text-bold text-lg">Overview</h1>
            <p>
                Pasta is another pastebin-like alternative.  It has support for
                various languages, End-To-End encryption, and link shortening.
            </p>
        </div>
        <div class="flex flex-col gap-2">
            <h1 class="text-bold text-lg">End-To-End Encryption</h1>
            <p>
                Pasta supports end-to-end encryption.  Simply enter a password
                before saving and that data will be encrypted in your browser
                before sent to the server.  You can then go to the provided
                link, which will prompt for you to enter the password again to
                unlock your pasta.
            </p>
            <p>
                Because the data never leaves your browser without being
                encrypted, the server will not know the content (or even the
                type) of your Pasta.
            </p>
        </div>
        <div class="flex flex-col gap-2">
            <h1 class="text-bold text-lg">Link Shortening</h1>
            <p>
                Pasta can also function as a link shortener, by using the
                <Button><Link /> New Link</Button> button in the header.
            </p>
            <p>
                This may also be combined with the password field in order to
                make the link encrypted, just keep in mind that that only
                encrypts the link itself and not the content behind it!
            </p>
        </div>
        <Dialog.Footer class="p-2">
            <div class="w-full flex flex-row justify-between">
                <Button href="https://github.com/funnyboy-roks/pasta">
                    <Git /> View Source
                </Button>
                <Dialog.Close class={buttonVariants({ variant: 'outline' })}>
                    Done
                </Dialog.Close>
            </div>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
