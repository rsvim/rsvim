set tabstop=2
set softtabstop=2
set shiftwidth=2
let g:rust_recommended_style = 0

au FileType rust setlocal colorcolumn=80
au FileType rust setlocal textwidth=79

au BufRead,BufNewFile *.txt setlocal colorcolumn=80
