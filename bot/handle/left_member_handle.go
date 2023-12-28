package handle

import (
	"arknights_bot/bot/utils"
	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api/v5"
)

func LeftMemberHandle(update tgbotapi.Update) (bool, error) {
	message := update.Message
	chatId := message.Chat.ID
	messageId := message.MessageID
	delMsg := tgbotapi.NewDeleteMessage(chatId, messageId)
	utils.DeleteMessage(delMsg)
	return true, nil
}
