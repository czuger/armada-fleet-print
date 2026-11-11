# Deploy shortcuts. Everything real lives in deploy/ — see deploy/config.sh. The
# server settings come from .env (copy .env.example), and the environment wins:
#
#   SSH_HOST=user@other-box make deploy
#
.PHONY: help setup nginx deploy rollback logs status versions test

# Runs a remote.sh subcommand with the settings from deploy/config.sh.
REMOTE = bash -c 'source deploy/config.sh && upload_remote_script && remote "$$0"'

help:  ## Affiche cette aide
	@grep -E '^[a-z-]+:.*?## ' $(MAKEFILE_LIST) | awk -F':.*?## ' '{printf "  %-10s %s\n", $$1, $$2}'

setup:  ## Préparation unique du serveur (dossiers, prérequis, config nginx)
	@./deploy/setup_server.sh

nginx:  ## Réinstalle la config nginx et recharge le proxy (sans déployer)
	@./deploy/nginx.sh

deploy:  ## Compile avec cross, envoie et démarre la nouvelle version
	@./deploy/deploy.sh

rollback:  ## Revient à la version précédente
	@./deploy/rollback.sh

logs:  ## Suit le journal de l'application (Ctrl-C pour arrêter)
	@bash -c 'source deploy/config.sh && ssh -t "$$SSH_HOST" journalctl --user -u "$$SERVICE_NAME" -f -n 100'

status:  ## État du service, du socket et de sa visibilité par nginx
	@$(REMOTE) status

versions:  ## Liste les versions conservées sur le serveur
	@$(REMOTE) versions

test:  ## Lance les tests en local
	@cargo test
