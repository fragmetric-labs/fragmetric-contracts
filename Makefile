wallet:
	aws secretsmanager get-secret-value --secret-id wallet --region ap-northeast-2 --output json | jq .SecretString | jq -c fromjson | jq .devnet | jq -c fromjson > id.json
