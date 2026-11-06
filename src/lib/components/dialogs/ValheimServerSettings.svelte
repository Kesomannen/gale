<script module lang="ts">
	import { m as moduleMessages } from '$lib/paraglide/messages';
	import type { Game, ValheimServerSettings as ValheimSettingsData } from '$lib/types';

	const DEFAULT_PORT = 2456;

	export function createValheimServerSettings(game: Game | null): ValheimSettingsData {
		return {
			type: 'valheim',
			serverName: moduleMessages.dedicatedServerSettings_defaultName({
				game: game?.name ?? moduleMessages.unknown()
			}),
			world: moduleMessages.dedicatedServerSettings_defaultWorld(),
			port: game?.dedicatedServer?.defaultPort ?? DEFAULT_PORT,
			publicServer: true,
			crossplay: false
		};
	}
</script>

<script lang="ts">
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Info from '$lib/components/ui/Info.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import { m } from '$lib/paraglide/messages';
	import { pushToast } from '$lib/toast';
	import type { ValheimServerSettings as ValheimSettings } from '$lib/types';

	const MAX_PORT = 65535;

	type Props = {
		settings: ValheimSettings;
		password: string;
	};

	let { settings = $bindable(), password = $bindable() }: Props = $props();

	export function prepare() {
		if (!Number.isInteger(settings.port) || settings.port < 1 || settings.port > MAX_PORT) {
			pushToast({
				type: 'error',
				message: m.dedicatedServerDialog_portError({
					label: m.dedicatedServerDialog_serverPort()
				})
			});
			return false;
		}

		settings.serverName = settings.serverName.trim();
		settings.world = settings.world.trim();
		return true;
	}
</script>

<div class="flex flex-col gap-2">
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
</div>
