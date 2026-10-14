<script lang="ts">
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Info from '$lib/components/ui/Info.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { LocalServerSettings } from '$lib/types';

	type Props = {
		settings: LocalServerSettings;
		password: string;
	};

	let { settings = $bindable(), password = $bindable() }: Props = $props();
</script>

<div class="mt-4 flex flex-col gap-2">
	<div class="flex items-center">
		<Label>{m.dedicatedServerDialog_serverName()}</Label>
		<InputField bind:value={settings.serverName} />
	</div>

	<div class="flex items-center">
		<Label>{m.dedicatedServerDialog_password()}</Label>
		<Info>{m.dedicatedServerDialog_credentialInfo()}</Info>
		<InputField
			bind:value={password}
			type="password"
			placeholder={m.dedicatedServerDialog_savedPassword()}
		/>
	</div>

	<div class="flex items-center">
		<Label>{m.dedicatedServerDialog_world()}</Label>
		<InputField bind:value={settings.world} />
	</div>

	<div class="flex items-center">
		<Label>{m.dedicatedServerDialog_serverPort()}</Label>
		<InputField
			bind:value={() => String(settings.port), (value) => (settings.port = Number(value))}
			inputmode="numeric"
		/>
	</div>

	<div class="flex items-center">
		<Label>{m.dedicatedServerDialog_public()}</Label>
		<Info>{m.dedicatedServerDialog_publicInfo()}</Info>
		<Checkbox bind:checked={settings.publicServer} />
	</div>

	<div class="flex items-center">
		<Label>{m.dedicatedServerDialog_crossplay()}</Label>
		<Info>{m.dedicatedServerDialog_crossplayInfo()}</Info>
		<Checkbox bind:checked={settings.crossplay} />
	</div>

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
