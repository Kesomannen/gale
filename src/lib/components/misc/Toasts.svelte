<script lang="ts">
	import { clearToast, pushInfoToast, toasts, type Toast } from '$lib/toast';
	import Icon from '@iconify/svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import clsx from 'clsx';
	import { expoOut } from 'svelte/easing';
	import { fade, slide } from 'svelte/transition';
	import IconButton from '../ui/IconButton.svelte';

	async function copyError(toast: Toast) {
		await writeText(`${toast.name ? `${toast.name}: ` : ''}${toast.message}`);
		pushInfoToast({
			message: 'Error copied to clipboard.'
		});
	}
</script>

<div class="absolute bottom-0 z-50 flex w-full flex-col items-end gap-1 p-2">
	{#each $toasts as toast, i (toast.id)}
		<div
			class="bg-primary-800 border-primary-700 relative max-w-3xl overflow-hidden rounded-md border shadow-xl"
			in:slide={{ duration: 150, easing: expoOut }}
			out:fade={{ duration: 50 }}
		>
			<div
				class={[
					toast.type === 'error' ? 'bg-red-600' : 'bg-accent-600',
					'absolute left-0 h-full w-1.5'
				]}
			></div>

			<div class="flex items-center p-2">
				<Icon
					class={clsx(
						toast.type === 'error' ? 'text-red-600' : 'text-accent-600',
						'mx-2 shrink-0 text-xl'
					)}
					icon={toast.type === 'error' ? 'mdi:error' : 'mdi:info-circle'}
				/>

				<div class="mr-4 grow overflow-hidden">
					{#if toast.name}
						<span class="text-primary-300">{toast.name}:</span>
					{/if}

					<span class="text-primary-100 font-semibold break-words">{toast.message}</span>
				</div>

				{#if toast.type === 'error'}
					<IconButton icon="mdi:content-copy" label="Copy error" onclick={() => copyError(toast)} />
				{/if}

				<IconButton icon="mdi:close" label="Clear toast" onclick={() => clearToast(i)} />
			</div>
		</div>
	{/each}
</div>
