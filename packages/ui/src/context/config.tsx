import React from 'react'
import { createContext, useContext } from 'react'
import Arrow from 'component/arrow'
import { Flag } from '@riezler/auth-sdk'

type $AuthConfig = {
	tos: string;
	privacy: string;
	Arrow: JSX.Element
}

export let DefaultConfig = {
	tos: '',
	privacy: '',
	Arrow: <Arrow />
}

export let AuthConfig = createContext<$AuthConfig>(DefaultConfig)

export function useConfig(): $AuthConfig {
	return useContext(AuthConfig)
}

export let FlagsCtx = createContext<Array<Flag>>([])

export function useFlags() {
	return useContext(FlagsCtx)
}