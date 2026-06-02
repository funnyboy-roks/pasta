<script lang="ts">
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
    import * as Select from '$lib/components/ui/select';

    import CodeMirror from 'svelte-codemirror-editor';
    import { oneDark } from '@codemirror/theme-one-dark';

    import LinkIcon from '@lucide/svelte/icons/link';
    import SaveIcon from '@lucide/svelte/icons/save';
    import PencilIcon from '@lucide/svelte/icons/pencil';
    import EyeIcon from '@lucide/svelte/icons/eye';
    import EyeOffIcon from '@lucide/svelte/icons/eye-off';
    import CircleQuestionMarkIcon from '@lucide/svelte/icons/circle-question-mark';

    import PasswordPrompt from '$lib/components/PasswordPrompt.svelte';
    import RedirectPrompt from '$lib/components/RedirectPrompt.svelte';
    import LinkCreation from '$lib/components/LinkCreation.svelte';

    import { PUBLIC_PASTA_UI_API } from '$env/static/public';
    import { supported_langs } from '$lib/lang';
    import { decrypt } from '$lib/crypto';
    import { toast } from 'svelte-sonner';
    import { hash, on_hash, set_hash } from '$lib/store';
    import { ENCRYPTION_TYPE, save_content, type EncryptedContent } from '$lib/save';
    import About from '$lib/components/About.svelte';

    let content = $state('');
    let password = $state('');
    let language = $state('plain');

    let show_password = $state(false);

    let saving = $state(false);

    let password_prompt = $state<PasswordPrompt>();
    let redirect = $state('');
    let create_link = $state(false);
    let show_about = $state(false);

    const save = () => {
        const save_inner = async () => {
            saving = true;

            const content_type = `text/${language}; charset=utf-8`;

            const res = await save_content(content, password, content_type);

            saving = false;

            switch (res.status) {
                case 'ok': {
                    set_hash(res.slug);
                    return res.link;
                }
                case 'error': {
                    throw res.message;
                }
                default: {
                    throw new Error('Uknown res.status');
                }
            }
        };

        toast.promise(save_inner, {
            loading: 'Saving...',
            success: (link) => `Saved at ${link}\nCopied to clipboard`,
            error: (reason) => `Error saving pasta:\n${reason}`,
        });
    };

    const load_hash = async (hash: string) => {
        if (!hash) return;
        if (password_prompt === undefined) throw new Error('password_prompt === undefined');

        const res = await fetch(`${PUBLIC_PASTA_UI_API}/${hash}?redirect=false`, {
            method: 'GET',
        });

        if (res.status === 404) {
            toast.error('Pasta expired or not found');
            set_hash('');
            return;
        }

        const content_type = res.headers.get('content-type');

        const encrypted = content_type === ENCRYPTION_TYPE;

        const lang_from_type = (
            content_type: string | null
        ): null | keyof typeof supported_langs => {
            if (!content_type || !content_type.startsWith('text/')) return null;

            const [lang] = content_type.substring('text/'.length).split('; ');
            if (supported_langs[lang]) {
                return lang;
            } else {
                return null;
            }
        };

        if (content_type === 'application/link') {
            const url = await res.text();
            redirect = url;
        } else if (encrypted) {
            const text = await res.text();
            const body = await password_prompt.prompt<EncryptedContent>(async (password) => {
                try {
                    return JSON.parse(await decrypt(text, password));
                } catch {
                    return null;
                }
            });

            if (body) {
                if (body.content_type === 'application/link') {
                    redirect = atob(body.content);
                } else {
                    language = lang_from_type(body.content_type) ?? '';
                    content = atob(body.content);
                }
            } else {
                set_hash('');
                content = '';
                language = '';
            }
        } else {
            language = lang_from_type(content_type) ?? '';
            content = await res.text();
        }
    };
    on_hash(load_hash);
</script>

<svelte:head>
    <title>Pasta {$hash ? `| ${$hash}` : ''}</title>
</svelte:head>

<PasswordPrompt bind:this={password_prompt} />
<RedirectPrompt {redirect} />
<LinkCreation bind:open={create_link} />
<About bind:open={show_about} />

<div class="flex h-screen max-h-screen flex-col">
    <div class="flex flex-row items-center justify-between bg-secondary">
        <div class="flex flex-row items-center">
            <Button variant="default" onclick={() => (create_link = true)}>
                <LinkIcon /> New Link
            </Button>
            <Button href="/">
                <PencilIcon /> New
            </Button>
        </div>
        <div class="flex-start flex flex-row items-center gap-1">
            <Select.Root type="single" name="language" bind:value={language}>
                <Select.Trigger class="w-[180px]">
                    {supported_langs[language]?.name ?? 'Language'}
                </Select.Trigger>
                <Select.Content>
                    {#each Object.entries(supported_langs) as [key, lang] (key)}
                        <Select.Item value={key} label={lang.name}>
                            {lang.name}
                        </Select.Item>
                    {/each}
                </Select.Content>
            </Select.Root>
            <div class="flex w-xs flex-row gap-1">
                <Input
                    type={show_password ? 'text' : 'password'}
                    placeholder="Password (optional)"
                    bind:value={password}
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
                <Button onclick={save} disabled={saving}>
                    <SaveIcon />
                    {saving ? 'Saving...' : 'Save'}
                </Button>
                <Button size="icon" variant="ghost" onclick={() => show_about = true} title="About">
                    <CircleQuestionMarkIcon class="scale-125" />
                </Button>
            </div>
        </div>
    </div>
    <div class="flex grow flex-col">
        <CodeMirror
            bind:value={content}
            class="flex grow"
            styles={{
                '&': {
                    flexGrow: 1,
                },
            }}
            theme={oneDark}
            lang={supported_langs[language]?.language()}
        />
    </div>
</div>
