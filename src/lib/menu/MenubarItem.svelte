<script lang="ts">
	import { Menubar } from 'bits-ui';
</script>

<Menubar.Item
	class="mx-1 cursor-default rounded-md py-1 pl-4 pr-12 text-left text-slate-400 hover:bg-gray-700 hover:text-slate-200"
	on:click
>
	<slot />
</Menubar.Item>
