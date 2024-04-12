rurl=${1-:rurl}
coproc $rurl
RURL_PID=$COPROC_PID
RURL_IN=${COPROC[1]}
RURL_OUT=${COPROC[0]}

##
## Kill the HTTP requester on exit
##
function _terminate() {
	if [[ "$RURL_PID" ]]; then
		kill $RURL_PID
		RURL_PID=""
	fi
}

trap "_terminate" EXIT ERR

function get_next() {
	echo "http://localhost:7000/next
 output_file: RURL_OUT.txt
 header_file: RURL_HEADERS.txt
 fields: content-length
" >&"${RURL_IN}"
	IFS= read -ru ${COPROC[0]} output
	[[ "$output" != "2"* ]] && { echo "FAILED on get next: $output"; exit 1; }
	echo $output
}

function send_response() {
	echo "http://localhost:7000/response
 input_file: RURL_OUT.txt
 header: Content-Type: application/json
 method: POST
" >&"${RURL_IN}"
	IFS= read -ru ${COPROC[0]} output
	[[ "$output" != "2"* ]] && { echo "FAILED on send response: $output"; exit 1; }
	echo $output
}

while IFS= read -r line
do
	get_next
	send_response
done

echo Done

exit