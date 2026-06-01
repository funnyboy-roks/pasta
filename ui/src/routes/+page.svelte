<script lang="ts">
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
	import * as Select from '$lib/components/ui/select';

    import CodeMirror from 'svelte-codemirror-editor';
    import { oneDark } from '@codemirror/theme-one-dark';

    import SaveIcon from '@lucide/svelte/icons/save';
    import PencilIcon from '@lucide/svelte/icons/pencil';
    import EyeIcon from '@lucide/svelte/icons/eye';
    import EyeOffIcon from '@lucide/svelte/icons/eye-off';

    import { PUBLIC_PASTA_UI_API } from '$env/static/public';
	import { supported_langs } from '$lib/lang';
	import { decrypt, encrypt, type EncryptedContent } from '$lib/crypto';
	import PasswordPrompt from '$lib/components/PasswordPrompt.svelte';
	import { toast } from 'svelte-sonner';
	import { hash, on_hash, set_hash } from '$lib/store';
	import { compress } from '$lib/save';

    let content = $state('');
    let password = $state('');
    let language = $state('');

    let show_password = $state(false);

    let saving = $state(false);

    let password_prompt = $state<PasswordPrompt>();

    const ENCRYPTION_TYPE = 'application/aes256gcm-encrypted';

    const save = () => {
        const saveInner = async () => {
            saving = true;

            const encrypted = !!password;

            const content_type = language ? `text/${language}; charset=utf-8` : 'text/plain; charset=utf-8';

            const body = encrypted
                ? await encrypt(JSON.stringify(<EncryptedContent>{
                    content_type,
                    content: btoa(content),
                }), password)
                : content;

            let res = await fetch(`${PUBLIC_PASTA_UI_API}/`, {
                method: 'POST',
                headers: {
                    'content-type': encrypted ? ENCRYPTION_TYPE : content_type,
                    'content-encoding': 'gzip',
                },
                body: await compress(body),
            });

            saving = false;

            if (res.status !== 201) {
                switch (res.status) {
                    case 409: throw 'Slug already exists';
                    case 413: throw 'Payload too large';
                }
            }

            const slug = await res.text();

            set_hash(slug);
            const link = `${location.origin}/#${slug}`;
            navigator.clipboard.writeText(link);

            return link;
        };

        toast.promise(saveInner, {
            loading: 'Saving...',
            success: (link) => `Saved at ${link}\nCopied to clipboard`,
            error: (reason) => `Error saving pasta:\n${reason}`,
        })
    };

    const load_hash = async (hash: string) => {
        if (!hash) return;

        let res = await fetch(`${PUBLIC_PASTA_UI_API}/${hash}`, {
            method: 'GET',
        });

        if (res.status == 404) {
            toast.error('Pasta expired or not found');
            set_hash('');
            return;
        }

        let content_type = res.headers.get('content-type');

        const encrypted = content_type === ENCRYPTION_TYPE;

        const lang_from_type = (content_type: string | null): null | keyof typeof supported_langs => {
            if (!content_type || !content_type.startsWith('text/')) return null;

            let lang = content_type.substring('text/'.length).split('; ')[0];
            if (supported_langs[lang]) {
                return lang;
            } else {
                return null
            }
        };

        if (encrypted) {
            const text = await res.text();
            const encrypted = await password_prompt!.prompt<EncryptedContent>(async (password) => {
                try {
                    return JSON.parse(await decrypt(text, password));
                } catch {
                    return null;
                };
            });
            if (encrypted) {
                language = lang_from_type(encrypted.content_type) ?? '';
                content = atob(encrypted.content);
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
    <title>Pasta { $hash ? `| ${$hash}` : '' }</title>
</svelte:head>

<PasswordPrompt bind:this={password_prompt} />

<div class="h-screen max-h-screen flex flex-col">
    <div class="bg-secondary flex flex-row items-center justify-between">
        <div class="flex flex-row items-center">
            <Button href="/" variant="ghost">
                <PencilIcon /> New
            </Button>
            <Button onclick={save} disabled={saving} variant="ghost">
                <SaveIcon /> { saving ? 'Saving...' : 'Save' }
            </Button>
            <div class="flex flex-row gap-1 w-xs">
                <Input type={show_password ? 'text' : 'password'} placeholder="Password (optional)" bind:value={password} />
                <Button variant="outline" size="icon" onclick={() => show_password = !show_password}>
                    {#if show_password}
                        <EyeIcon />
                    {:else}
                        <EyeOffIcon />
                    {/if}
                </Button>
            </div>
        </div>
        <div class="flex-start flex flex-row items-center">
            <Select.Root type="single" name="language" bind:value={language}>
                <Select.Trigger class="w-[180px]">
                    { supported_langs[language]?.name ?? "Language" }
                </Select.Trigger>
                <Select.Content>
                    {#each Object.entries(supported_langs) as [key, lang] (key)}
                        <Select.Item value={key} label={lang.name}>
                            {lang.name}
                        </Select.Item>
                    {/each}
                </Select.Content>
            </Select.Root>
        </div>
    </div>
    <div class="grow flex flex-col">
            <CodeMirror
                bind:value={content}
                class="grow flex"
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
