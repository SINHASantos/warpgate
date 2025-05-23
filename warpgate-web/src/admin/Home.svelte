<script lang="ts">
    import Fa from 'svelte-fa'
    import { faCircleDot as iconActive } from '@fortawesome/free-regular-svg-icons'
    import { onDestroy } from 'svelte'
    import { link } from 'svelte-spa-router'
    import { api, type SessionSnapshot } from 'admin/lib/api'
    import { formatDistance } from 'date-fns'
    import { timer, Observable, switchMap, from, combineLatest, fromEvent, merge } from 'rxjs'
    import RelativeDate from './RelativeDate.svelte'
    import AsyncButton from 'common/AsyncButton.svelte'
    import ItemList, { type LoadOptions, type PaginatedResponse } from 'common/ItemList.svelte'
    import { Input } from '@sveltestrap/sveltestrap'
    import { autosave } from 'common/autosave'
    import GettingStarted from 'common/GettingStarted.svelte'
    import { serverInfo } from 'gateway/lib/store'

    let [showActiveOnly, showActiveOnly$] = autosave('sessions-list:show-active-only', false)
    let [showLoggedInOnly, showLoggedInOnly$] = autosave('sessions-list:show-logged-in-only', true)

    let activeSessionCount: number|undefined = $state()

    let socket = new WebSocket(`wss://${location.host}/@warpgate/admin/api/sessions/changes`)
    let sessionChanges$ = fromEvent(socket, 'message')
    onDestroy(() => socket.close())

    function loadSessions (opt: LoadOptions): Observable<PaginatedResponse<SessionSnapshot>> {
        return combineLatest([
            showActiveOnly$,
            showLoggedInOnly$,
            merge(timer(0, 60000), sessionChanges$),
        ]).pipe(switchMap(([activeOnly, loggedInOnly]) => {
            api.getSessions({
                activeOnly: true,
                limit: 1,
            }).then(response => {
                activeSessionCount = response.total
            })
            return from(api.getSessions({
                activeOnly,
                loggedInOnly,
                ...opt,
            }))
        }))
    }

    async function _reloadSessions (): Promise<void> {
        activeSessionCount = (await api.getSessions({ activeOnly: true })).total
    }

    async function closeAllSesssions () {
        await api.closeAllSessions()
    }

    function describeSession (session: SessionSnapshot): string {
        let user = session.username ?? (session.ended ? '<not logged in>' : '<logging in>')
        if (!session.target) {
            return user
        }
        let target = session.target.name
        return `${user} on ${target}`
    }

    _reloadSessions()
    const interval = setInterval(_reloadSessions, 1000000)
    onDestroy(() => clearInterval(interval))
</script>

{#if $serverInfo?.setupState}
    <GettingStarted
        setupState={$serverInfo?.setupState} />
{/if}

{#if activeSessionCount !== undefined}
<div class="page-summary-bar">
    {#if activeSessionCount }
        <h1>
            <span>active sessions:</span> <span class="counter">{activeSessionCount}</span>
        </h1>
        <div class="ms-auto">
            <AsyncButton color="warning" click={closeAllSesssions}>
                Close all
            </AsyncButton>
        </div>
    {:else}
        <h1>no active sessions</h1>
    {/if}
</div>
{/if}

<ItemList load={loadSessions} pageSize={100}>
    {#snippet header()}
        <div  class="d-flex align-items-center mb-1 w-100">
            <div class="ms-auto"></div>
            <Input class="ms-3" type="switch" label="Active only" bind:checked={$showActiveOnly} />
            <Input class="ms-3" type="switch" label="Logged in only" bind:checked={$showLoggedInOnly} />
        </div>
    {/snippet}

    {#snippet item(session)}
        <a

            class="list-group-item list-group-item-action"
            href="/sessions/{session.id}"
            use:link>
            <div class="main">
                <div class="icon" class:text-success={!session.ended}>
                    {#if !session.ended}
                        <Fa icon={iconActive} fw />
                    {/if}
                </div>
                <div class="protocol text-muted me-2">{session.protocol}</div>
                <strong>
                    {describeSession(session)}
                </strong>

                <div class="meta">
                    {#if session.ended }
                        {formatDistance(new Date(session.started), new Date(session.ended))}
                    {/if}
                </div>

                <div class="meta ms-auto">
                    <RelativeDate date={session.started} />
                </div>
            </div>
        </a>
    {/snippet}
</ItemList>

<style lang="scss">
    .list-group-item {
        .icon {
            display: flex;
            align-items: center;
            margin-right: 5px;
            width: 20px;
        }

        .main {
            display: flex;
            align-items: center;
        }

        .protocol {
            min-width: 3.5rem;
        }

        .meta {
            opacity: .75;
            margin-left: 25px;
            font-size: .75rem;
        }
    }
</style>
