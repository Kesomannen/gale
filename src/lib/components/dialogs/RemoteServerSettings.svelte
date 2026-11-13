<script lang="ts">
	import Info from '$lib/components/ui/Info.svelte';
	import InfoBox from '$lib/components/ui/InfoBox.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import PathField from '$lib/components/ui/PathField.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { RemoteProtocol, RemoteServerSettings } from '$lib/types';
	import { open } from '@tauri-apps/plugin-dialog';

	const DEFAULT_SFTP_PORT = 22;
	const DEFAULT_FTP_PORT = 21;

	type Props = {
		settings: RemoteServerSettings;
		credential: string;
	};

	let { settings = $bindable(), credential = $bindable() }: Props = $props();

	function clearTrust() {
		settings.trustedHostKey = null;
		settings.trustedInvalidCertificateHost = null;
	}

	function changeProtocol(protocol: RemoteProtocol) {
		if (
			(settings.protocol === 'sftp' && settings.port === DEFAULT_SFTP_PORT) ||
			(settings.protocol !== 'sftp' && settings.port === DEFAULT_FTP_PORT)
		) {
			settings.port = protocol === 'sftp' ? DEFAULT_SFTP_PORT : DEFAULT_FTP_PORT;
		}

		settings.protocol = protocol;
		clearTrust();
	}

	function changeHost(host: string) {
		if (host !== settings.host) clearTrust();
		settings.host = host;
	}

	function changePort(value: string) {
		const port = Number(value);
		if (port !== settings.port) clearTrust();
		settings.port = port;
	}

	async function choosePrivateKey() {
		const selected = await open({
			title: m.dedicatedServerDialog_privateKeyTitle(),
			directory: false,
			multiple: false
		});
		if (typeof selected === 'string') settings.privateKeyPath = selected;
	}
</script>

<div class="mt-4 flex flex-col gap-2">
	<InfoBox type={settings.protocol === 'ftp' ? 'warning' : 'info'}>
		{settings.protocol === 'ftp'
			? m.dedicatedServerDialog_ftpInfo()
			: m.dedicatedServerDialog_sftpInfo()}
	</InfoBox>

	<div class="flex items-center">
		<Label>{m.dedicatedServerDialog_protocol()}</Label>
		<Select
			type="single"
			triggerClass="grow"
			value={settings.protocol}
			onValueChange={(value) => changeProtocol(value as RemoteProtocol)}
			items={[
				{ value: 'sftp', label: m.dedicatedServerDialog_protocolSftp() },
				{ value: 'ftp', label: m.dedicatedServerDialog_protocolFtp() }
			]}
		/>
	</div>

	<div class="flex items-center">
		<Label>{m.dedicatedServerDialog_host()}</Label>
		<InputField bind:value={() => settings.host, changeHost} placeholder="example.com" />
	</div>

	<div class="flex items-center">
		<Label>
			{settings.protocol === 'sftp'
				? m.dedicatedServerDialog_sshPort()
				: m.dedicatedServerDialog_ftpPort()}
		</Label>
		<InputField bind:value={() => String(settings.port), changePort} inputmode="numeric" />
	</div>

	<div class="flex items-center">
		<Label>{m.dedicatedServerDialog_username()}</Label>
		<InputField bind:value={settings.username} />
	</div>

	{#if settings.protocol === 'sftp'}
		<div class="flex items-center">
			<Label>{m.dedicatedServerDialog_authentication()}</Label>
			<Select
				type="single"
				triggerClass="grow"
				bind:value={settings.authentication}
				items={[
					{ value: 'password', label: m.dedicatedServerDialog_password() },
					{ value: 'privateKey', label: m.dedicatedServerDialog_privateKeyFile() },
					{ value: 'agent', label: m.dedicatedServerDialog_sshAgent() }
				]}
			/>
		</div>
	{/if}

	{#if settings.protocol === 'sftp' && settings.authentication === 'privateKey'}
		<PathField
			label={m.dedicatedServerDialog_privateKey()}
			bind:value={settings.privateKeyPath}
			onclick={choosePrivateKey}
			icon="mdi:file-key"
		>
			{m.dedicatedServerDialog_privateKeyInfo()}
		</PathField>
	{/if}

	{#if settings.protocol !== 'sftp' || settings.authentication !== 'agent'}
		<div class="flex items-center">
			<Label>
				{settings.protocol !== 'sftp' || settings.authentication === 'password'
					? m.dedicatedServerDialog_password()
					: m.dedicatedServerDialog_keyPassphrase()}
			</Label>
			<Info>{m.dedicatedServerDialog_credentialInfo()}</Info>
			<InputField
				bind:value={credential}
				type="password"
				placeholder={settings.protocol !== 'sftp' || settings.authentication === 'password'
					? m.dedicatedServerDialog_savedPassword()
					: m.dedicatedServerDialog_remoteSavedPassphrase()}
			/>
		</div>
	{/if}

	<div class="flex items-center">
		<Label>{m.dedicatedServerDialog_directory()}</Label>
		<InputField bind:value={settings.serverDirectory} placeholder="/home/valheim/server" />
	</div>
</div>
