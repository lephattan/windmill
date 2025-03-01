<script lang="ts">
	import { getContext } from 'svelte'
	import { initConfig, initOutput } from '../../editor/appUtils'
	import type { AppInput } from '../../inputType'
	import type { AppViewerContext, ComponentCustomCSS, RichConfigurations } from '../../types'
	import { initCss } from '../../utils'
	import RunnableWrapper from '../helpers/RunnableWrapper.svelte'
	import Markdown from 'svelte-exmarkdown'
	import { classNames } from '$lib/utils'
	import { components } from '../../editor/component'
	import ResolveConfig from '../helpers/ResolveConfig.svelte'
	import ResolveStyle from '../helpers/ResolveStyle.svelte'
	import { gfmPlugin } from 'svelte-exmarkdown/gfm'
	export let id: string
	export let componentInput: AppInput | undefined
	export let initializing: boolean | undefined = undefined
	export let customCss: ComponentCustomCSS<'mardowncomponent'> | undefined = undefined
	export let render: boolean
	export let configuration: RichConfigurations

	const { app, worldStore, mode } = getContext<AppViewerContext>('AppViewerContext')

	const resolvedConfig = initConfig(
		components['mardowncomponent'].initialData.configuration,
		configuration
	)

	const outputs = initOutput($worldStore, id, {
		result: undefined,
		loading: false
	})

	let result: string | undefined = undefined

	let css = initCss($app.css?.mardowncomponent, customCss)

	const proseMapping = {
		sm: 'prose-sm',
		Default: 'prose-base',
		lg: 'prose-lg',
		xl: 'prose-xl',
		'2xl': 'prose-2xl'
	}
</script>

{#each Object.keys(components['mardowncomponent'].initialData.configuration) as key (key)}
	<ResolveConfig
		{id}
		{key}
		bind:resolvedConfig={resolvedConfig[key]}
		configuration={configuration[key]}
	/>
{/each}

{#each Object.keys(css ?? {}) as key (key)}
	<ResolveStyle
		{id}
		{customCss}
		{key}
		bind:css={css[key]}
		componentStyle={$app.css?.mardowncomponent}
	/>
{/each}

{#if render}
	<div
		on:pointerdown={(e) => {
			if ($mode != 'preview') {
				e?.preventDefault()
			}
		}}
		class={classNames(
			'h-full w-full overflow-y-auto prose max-w-full',
			resolvedConfig?.size ? proseMapping[resolvedConfig.size] : '',
			css?.container?.class,
			' dark:prose-invert',
			'wm-markdown'
		)}
		style={css?.container?.style}
	>
		<RunnableWrapper
			{outputs}
			render={true}
			autoRefresh
			{componentInput}
			{id}
			bind:initializing
			bind:result
		>
			{#if result}
				{#key result}
					<Markdown md={result} plugins={[gfmPlugin()]} />
				{/key}
			{/if}
		</RunnableWrapper>
	</div>
{:else}
	<RunnableWrapper {outputs} render={false} autoRefresh {componentInput} {id} />
{/if}
