" Initialize the channel
if !exists('s:cobolJobId')
	let s:cobolJobId = 0
endif

" Path
let s:scriptdir = resolve(expand('<sfile>:p:h') . '/..')
let s:bin = s:scriptdir . '/target/release/neovim-cobol'

" Constants for RPC messages.
let s:WriteWorkingTexteFD = 'write_working_texte_fd'

" Entry point. Initialize RPC. If it succeeds, then attach commands to the `rpcnotify` invocations.
function! s:connect()
  let id = s:initRpc()
  
  if 0 == id
    echoerr "cobol: cannot start rpc process"
  elseif -1 == id
    echoerr "cobol: rpc process is not executable"
  else
    " Mutate our jobId variable to hold the channel ID
    let s:cobolJobId = id
    
    call s:configureCommands()
  endif
endfunction

function! s:configureCommands()
  command! -nargs=1 WriteWorkingTexteFD :call s:write_working_texte_fd(<args>)
endfunction

function! s:write_working_texte_fd(serialized)
  call rpcnotify(s:cobolJobId, s:WriteWorkingTexteFD, a:serialized)
endfunction

" Initialize RPC
function! s:initRpc()
  if s:cobolJobId == 0
    let jobid = jobstart([s:bin], { 'rpc': v:true })
    return jobid
  else
    return s:cobolJobId
  endif
endfunction

call s:connect()

" copy paste this
:function TexteCobol()
	let _type_texte = input('Insert standard of texte: 0: TEXTE-..., 1: RAPP-..., 2: CSV-..., empty: go to next step: ')
	let _texte_cobol = ''
	if len(_type_texte) == 0
		let _texte_cobol = input('Insert variable ...: ')
	elseif _type_texte == 0
		let _texte_cobol = 'TEXTE-'
		let _texte_cobol = input('Insert variable '._texte_cobol.'-...: ')
		let _texte_cobol = 'TEXTE-'._texte_cobol
	elseif _type_texte == 1
		let _texte_cobol = 'RAPP-'
		let _texte_cobol = input('Insert variable '._texte_cobol.'-...: ')
		let _texte_cobol = 'RAPP-'._texte_cobol
	elseif _type_texte == 2
		let _texte_cobol = 'CSV-'
		let _texte_cobol = input('Insert variable '._texte_cobol.'-...: ')
		let _texte_cobol = 'CSV-'._texte_cobol
	else
		let _texte_cobol = input('Insert variable ...: ')
	endif
	let texte_f = input('Insert french text of '.toupper(_texte_cobol).': ')
	let texte_d = input('Insert german text of '.toupper(_texte_cobol).' (french: '.texte_f.'): ')
	let [ bufnum, lnum, column, off ] = getpos('.')
	let serialized = json_encode({'buffer': bufnum, 'var_def': toupper(_texte_cobol), 'texte_f': texte_f, 'texte_d': texte_d})
	:WriteWorkingTexteFD serialized
:endfunction