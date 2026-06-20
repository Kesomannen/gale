<script lang="ts">
	import Info from '$lib/components/ui/Info.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import translation from '$lib/state/translation.svelte';
	import Icon from '@iconify/svelte';
	import { confirm } from '@tauri-apps/plugin-dialog';

	let prefs = $derived(translation.prefs);
	let enabled = $derived(prefs?.enabled ?? false);
	let apiUrl = $derived(prefs?.apiUrl ?? '');
	let apiKey = $derived(prefs?.apiKey ?? '');
	let model = $derived(prefs?.model ?? 'gpt-4o-mini');

	let testResult: { ok: boolean; message: string } | null = $state(null);
	let testing = $state(false);
	let showApiKey = $state(false);

	async function toggleEnabled() {
		if (!prefs) return;

		if (prefs.enabled) {
			const confirmed = await confirm('Disable AI translation?');
			if (!confirmed) return;
		}

		translation.prefs = { ...prefs, enabled: !prefs.enabled };
		await translation.savePrefs();
	}

	async function updateApiUrl(value: string) {
		if (!prefs) return;
		translation.prefs = { ...prefs, apiUrl: value };
		await translation.savePrefs();
	}

	async function updateApiKey(value: string) {
		if (!prefs) return;
		translation.prefs = { ...prefs, apiKey: value };
		await translation.savePrefs();
	}

	async function updateModel(value: string) {
		if (!prefs) return;
		translation.prefs = { ...prefs, model: value };
		await translation.savePrefs();
	}

	async function updateBatchSize(value: string) {
		if (!prefs) return;
		const num = parseInt(value, 10);
		if (isNaN(num) || num < 1) return;
		translation.prefs = { ...prefs, batchSize: num };
		await translation.savePrefs();
	}

	async function handleTest() {
		testing = true;
		testResult = null;
		try {
			testResult = await translation.testConnection();
		} finally {
			testing = false;
		}
	}
</script>

<div class="my-2 rounded-lg border border-primary-700 p-4">
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-2">
			<Icon icon="mdi:translate" class="text-primary-300 text-xl" />
			<Label>AI Translation</Label>
		</div>

		<button
			class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors"
			class:bg-primary-600={enabled}
			class:bg-primary-800={!enabled}
			onclick={toggleEnabled}
			aria-label="Toggle AI translation"
		>
			<span
				class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
				class:translate-x-6={enabled}
				class:translate-x-1={!enabled}
			></span>
		</button>
	</div>

	{#if enabled}
		<div class="mt-4 space-y-3">
			<div>
				<Label>API URL</Label>
				<input
					type="text"
					value={apiUrl}
					placeholder="https://api.openai.com"
					class="bg-primary-800 border-primary-700 text-primary-100 mt-1 w-full rounded-lg border px-3 py-2 text-sm"
					onchange={(e) => updateApiUrl(e.currentTarget.value)}
				/>
				<p class="text-primary-400 mt-1 text-xs">Base URL or full endpoint. Auto-appends /v1/chat/completions if needed.</p>
			</div>

			<div>
				<Label>API Key</Label>
				<div class="relative mt-1">
					<input
						type={showApiKey ? 'text' : 'password'}
						value={apiKey}
						placeholder="sk-..."
						class="bg-primary-800 border-primary-700 text-primary-100 w-full rounded-lg border px-3 py-2 pr-10 text-sm"
						onchange={(e) => updateApiKey(e.currentTarget.value)}
					/>
					<button
						type="button"
						class="text-primary-400 hover:text-primary-200 absolute right-2 top-1/2 -translate-y-1/2"
						onclick={() => (showApiKey = !showApiKey)}
						aria-label={showApiKey ? 'Hide API key' : 'Show API key'}
					>
						<Icon icon={showApiKey ? 'mdi:eye-off' : 'mdi:eye'} />
					</button>
				</div>
			</div>

			<div>
				<Label>Model</Label>
				<input
					type="text"
					value={model}
					placeholder="gpt-4o-mini"
					class="bg-primary-800 border-primary-700 text-primary-100 mt-1 w-full rounded-lg border px-3 py-2 text-sm"
					onchange={(e) => updateModel(e.currentTarget.value)}
				/>
			</div>

			<div>
				<Label>Batch Size</Label>
				<input
					type="number"
					value={prefs?.batchSize ?? 20}
					min="1"
					max="50"
					placeholder="20"
					class="bg-primary-800 border-primary-700 text-primary-100 mt-1 w-full rounded-lg border px-3 py-2 text-sm"
					onchange={(e) => updateBatchSize(e.currentTarget.value)}
				/>
				<p class="text-primary-400 mt-1 text-xs">Number of mods to translate per request (1-50)</p>
			</div>

			<button
				class="bg-accent-600 hover:bg-accent-500 flex items-center gap-2 rounded-md px-4 py-2 text-sm text-white disabled:opacity-50"
				onclick={handleTest}
				disabled={testing || !apiUrl || !apiKey}
			>
				{#if testing}
					<Icon icon="mdi:loading" class="animate-spin" />
					Testing...
				{:else}
					<Icon icon="mdi:connection" />
					Test Connection
				{/if}
			</button>

			{#if testResult}
				<div
					class="flex items-center gap-2 rounded-md p-2 text-sm"
					class:bg-green-900={testResult.ok}
					class:text-green-300={testResult.ok}
					class:bg-red-900={!testResult.ok}
					class:text-red-300={!testResult.ok}
				>
					<Icon icon={testResult.ok ? 'mdi:check-circle' : 'mdi:alert-circle'} />
					{testResult.message}
				</div>
			{/if}

			{#if translation.error}
				<div class="text-sm text-red-400">{translation.error}</div>
			{/if}

			<Info>
				Configure an OpenAI-compatible API endpoint for translating mod names and descriptions.
			</Info>
		</div>
	{/if}
</div>
