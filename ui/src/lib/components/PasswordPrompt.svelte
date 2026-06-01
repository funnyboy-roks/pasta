<script lang="ts">
    import * as Dialog from '$lib/components/ui/dialog';
    import { Input } from '$lib/components/ui/input';
    import { Button } from '$lib/components/ui/button';

    import Lock from '@lucide/svelte/icons/lock';
    import EyeIcon from '@lucide/svelte/icons/eye';
    import EyeOffIcon from '@lucide/svelte/icons/eye-off';

    let open = $state(false);
    let show_password = $state(false);
    let password = $state('');
    let error = $state('');
    let unlocking = $state(false);

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let resolve: ((t: any | null) => void) | undefined = undefined;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let on_passwd: ((password: string) => Promise<any | null>) | undefined = undefined;

    export const prompt = <T,>(
        on_password: (password: string) => Promise<T | null>
    ): Promise<T | null> =>
        new Promise((res) => {
            open = true;
            on_passwd = on_password;
            resolve = res;
        });

    const submit = async (e: SubmitEvent) => {
        e.preventDefault();
        if (!on_passwd || !resolve) throw new Error('submit called before prompt');

        unlocking = true;
        const valid = await on_passwd(password);
        unlocking = false;

        if (valid === null) {
            error = 'Invalid Password';
            password = '';
        } else {
            resolve?.(valid);
            resolve = undefined;
            on_passwd = undefined;
            open = false;
            error = '';
        }
    };

    const new_pasta = () => {
        if (!on_passwd || !resolve) throw new Error('new_pasta called before prompt');
        resolve?.(null);
        resolve = undefined;
        on_passwd = undefined;
        open = false;
        error = '';
    };
</script>

<Dialog.Root bind:open>
    <Dialog.Content
        showCloseButton={false}
        escapeKeydownBehavior="defer-otherwise-ignore"
        interactOutsideBehavior="defer-otherwise-ignore"
        trapFocus
        preventScroll
    >
        <Dialog.Header>
            <div class="my-5 flex w-full flex-row justify-center">
                <Lock size={96} />
            </div>
            <Dialog.Title>This Pasta is Locked with a Password!</Dialog.Title>
            <Dialog.Description>
                This pasta is locked with a password. Please enter the password in order to unlock
                the pasta.
            </Dialog.Description>
        </Dialog.Header>
        <form onsubmit={submit}>
            <div class="flex flex-col gap-1">
                <div class="flex flex-col gap-1">
                    <div class="flex flex-row gap-1">
                        <Input
                            type={show_password ? 'text' : 'password'}
                            autofocus
                            bind:value={password}
                            placeholder="Password"
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
                <div class="flex flex-row justify-between">
                    <Button
                        type="button"
                        class="w-1/3 self-end"
                        variant="secondary"
                        onclick={new_pasta}
                    >
                        New Pasta
                    </Button>
                    <Button type="submit" class="w-1/3 self-end" disabled={unlocking || !password}>
                        Unlock Pasta
                    </Button>
                </div>
            </div>
        </form>
    </Dialog.Content>
</Dialog.Root>
