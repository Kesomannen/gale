<script module lang="ts">
	import type { Game, LocalServerSettings as LocalSettingsData } from '$lib/types';
	import { createValheimServerSettings } from './ValheimServerSettings.svelte';

	export function createLocalServerSettings(game: Game | null): LocalSettingsData {
		switch (game?.slug) {
			case 'valheim':
			case undefined:
				return {
					game: createValheimServerSettings(game),
					extraArgs: ''
				};
			default:
				throw new Error(`unsupported dedicated server game: ${game?.slug ?? 'unknown'}`);
		}
	}
</script>

<script lang="ts">
	import InputField from '$lib/components/ui/InputField.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { LocalServerSettings as LocalSettings } from '$lib/types';
	import ValheimServerSettings from './ValheimServerSettings.svelte';

	type Props = {
		settings: LocalSettings;
		password: string;
	};

	let { settings = $bindable(), password = $bindable() }: Props = $props();
	let gameSettings = $state<ValheimServerSettings>();

	export function prepare() {
		settings.extraArgs = settings.extraArgs.trim();

		switch (settings.game.type) {
			case 'valheim':
				return gameSettings?.prepare() ?? false;
		}
	}
</script>

<div class="mt-4 flex flex-col gap-2">
	{#if settings.game.type === 'valheim'}
		<ValheimServerSettings bind:this={gameSettings} bind:settings={settings.game} bind:password />
	{/if}

	<details>
		<summary class="text-primary-600 dark:text-primary-300 cursor-pointer">
			{m.dedicatedServerDialog_advancedOptions()}
		</summary>
		<div class="mt-3 flex items-center">
			<Label>{m.dedicatedServerDialog_additionalArgs()}</Label>
			<InputField bind:value={settings.extraArgs} placeholder="-savedir ..." />
		</div>
	</details>
</div>
