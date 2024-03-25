package utils

import (
	bot "arknights_bot/config"
	"bytes"
	"context"
	"crypto/md5"
	"encoding/hex"
	"github.com/go-redis/redis/v8"
	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api/v5"
	gonanoid "github.com/matoous/go-nanoid/v2"
	"github.com/playwright-community/playwright-go"
	"github.com/tidwall/gjson"
	"golang.org/x/image/webp"
	"gorm.io/gorm"
	"image"
	"image/color"
	"image/png"
	"io"
	"log"
	"mime/multipart"
	"net/http"
	"strings"
	"time"
	"unicode/utf8"
)

var ctx = context.Background()

// GetFullName 获取用户全名
func GetFullName(user *tgbotapi.User) string {
	var buffer bytes.Buffer
	firstName := user.FirstName
	lastName := user.LastName
	if firstName != "" {
		buffer.WriteString(firstName)
	}
	if lastName != "" {
		buffer.WriteString(lastName)
	}
	return buffer.String()
}

type GroupInvite struct {
	Id           string    `json:"id" gorm:"primaryKey"`
	GroupName    string    `json:"groupName"`
	GroupNumber  int64     `json:"groupNumber"`
	UserName     string    `json:"userName"`
	UserNumber   int64     `json:"userNumber"`
	MemberName   string    `json:"memberName"`
	MemberNumber int64     `json:"memberNumber"`
	CreateTime   time.Time `json:"createTime" gorm:"autoCreateTime"`
	UpdateTime   time.Time `json:"updateTime" gorm:"autoUpdateTime"`
	Remark       string    `json:"remark"`
}

type GroupJoined struct {
	Id          string    `json:"id" gorm:"primaryKey"`
	GroupName   string    `json:"groupName"`
	GroupNumber int64     `json:"groupNumber"`
	News        int64     `json:"news"`
	CreateTime  time.Time `json:"createTime" gorm:"autoCreateTime"`
	UpdateTime  time.Time `json:"updateTime" gorm:"autoUpdateTime"`
	Remark      string    `json:"remark"`
}

// SaveInvite 保存邀请记录
func SaveInvite(message *tgbotapi.Message, member *tgbotapi.User) {
	id, _ := gonanoid.New(32)
	groupInvite := GroupInvite{
		Id:           id,
		GroupName:    message.Chat.Title,
		GroupNumber:  message.Chat.ID,
		UserName:     GetFullName(message.From),
		UserNumber:   message.From.ID,
		MemberName:   GetFullName(member),
		MemberNumber: member.ID,
	}

	bot.DBEngine.Table("group_invite").Create(&groupInvite)
}

// SaveJoined 保存入群记录
func SaveJoined(message *tgbotapi.Message) {
	id, _ := gonanoid.New(32)
	groupJoined := GroupJoined{
		Id:          id,
		GroupName:   message.Chat.Title,
		GroupNumber: message.Chat.ID,
		News:        0,
	}

	bot.DBEngine.Table("group_joined").Create(&groupJoined)
}

// GetJoinedByChatId 查询入群记录
func GetJoinedByChatId(chatId int64) *gorm.DB {
	return bot.DBEngine.Raw("select * from group_joined where group_number = ? limit 1", chatId)
}

// IsAdmin 是否管理员
func IsAdmin(chatId, userId int64) bool {
	getChatMemberConfig := tgbotapi.GetChatMemberConfig{
		ChatConfigWithUser: tgbotapi.ChatConfigWithUser{
			ChatID: chatId,
			UserID: userId,
		},
	}
	memberInfo, _ := bot.Arknights.GetChatMember(getChatMemberConfig)
	if memberInfo.Status != "creator" && memberInfo.Status != "administrator" {
		return false
	}
	return true
}

// DownloadFile 下载tg文件
func DownloadFile(fileId string) (io.ReadCloser, string) {
	fileUrl, _ := bot.Arknights.GetFileDirectURL(fileId)
	fileType := fileUrl[strings.LastIndex(fileUrl, ".")+1:]
	response, _ := http.Get(fileUrl)
	body := response.Body
	return body, fileType
}

// GetAccountByUserId 查询账号信息
func GetAccountByUserId(userId int64) *gorm.DB {
	return bot.DBEngine.Raw("select * from user_account where user_number = ?", userId)
}

// GetAccountByUserIdAndSklandId 查询账号信息
func GetAccountByUserIdAndSklandId(userId int64, sklandId string) *gorm.DB {
	return bot.DBEngine.Raw("select * from user_account where user_number = ? and skland_id = ?", userId, sklandId)
}

// GetAccountByUid 查询账号信息
func GetAccountByUid(userId int64, uid string) *gorm.DB {
	return bot.DBEngine.Raw("select t.* from user_account t, user_player t1 where t.id = t1.account_id and t.user_number = ? and t1.uid = ? limit 1", userId, uid)
}

// GetPlayersByUserId 查询绑定角色列表
func GetPlayersByUserId(userId int64) *gorm.DB {
	return bot.DBEngine.Raw("select * from user_player where user_number = ?", userId)
}

// GetBPlayersByUserId 查询绑定B服角色列表
func GetBPlayersByUserId(userId int64) *gorm.DB {
	return bot.DBEngine.Raw("select * from user_player where user_number = ? and server_name in('b服','bilibili服')", userId)
}

// GetPlayerByUserId 查询绑定角色
func GetPlayerByUserId(userId int64, uid string) *gorm.DB {
	return bot.DBEngine.Raw("select * from user_player where user_number = ? and uid = ?", userId, uid)
}

// GetAutoSign 查询自动签到用户
func GetAutoSign() *gorm.DB {
	return bot.DBEngine.Raw("select * from user_sign")
}

// GetAutoSignByUserId 查询自动签到用户
func GetAutoSignByUserId(userId int64) *gorm.DB {
	return bot.DBEngine.Raw("select * from user_sign where user_number = ?", userId)
}

// GetJoinedGroups 获取加入的群组
func GetJoinedGroups() []int64 {
	var groups []int64
	bot.DBEngine.Raw("select group_number from group_joined where news = 1 group by group_number").Scan(&groups)
	return groups
}

// GetUserGacha 获取角色抽卡记录
func GetUserGacha(userId int64, uid string) *gorm.DB {
	return bot.DBEngine.Raw("select * from user_gacha where user_number = ? and uid = ? order by ts desc, pool_order desc", userId, uid)
}

// GetUserPoolCount 获取角色卡池水位
func GetUserPoolCount(userId int64, uid string) *gorm.DB {
	return bot.DBEngine.Raw("select pool_name, count(1) pool_count, max(ts) ts from user_gacha where user_number = ? and uid = ? group by pool_name order by ts", userId, uid)
}

// RedisSet redis存值
func RedisSet(key string, val interface{}, expiration time.Duration) {
	err := bot.GoRedis.Set(ctx, key, val, expiration).Err()
	if err != nil {
		log.Println(err)
	}
}

// RedisGet redis取值
func RedisGet(key string) string {
	val, err := bot.GoRedis.Get(ctx, key).Result()
	if err != nil {
		if err == redis.Nil {
			return ""
		}
		log.Println(err)
	}
	return val
}

// RedisIsExists 判断redis值是否存在
func RedisIsExists(key string) bool {
	val := RedisGet(key)
	if val == "" {
		return false
	}
	return true
}

// RedisDel redis根据key删除
func RedisDel(key string) {
	err := bot.GoRedis.Del(ctx, key).Err()
	if err != nil {
		log.Println(err)
	}
}

// RedisScanKeys 扫描匹配keys
func RedisScanKeys(match string) (*redis.ScanIterator, context.Context) {
	return bot.GoRedis.Scan(ctx, 0, match, 0).Iterator(), ctx
}

// RedisSetList redis添加链表元素
func RedisSetList(key string, val interface{}) {
	err := bot.GoRedis.RPush(ctx, key, val).Err()
	if err != nil {
		log.Println(err)
	}
}

// RedisGetList redis获取所有链表元素
func RedisGetList(key string) []string {
	val, err := bot.GoRedis.LRange(ctx, key, 0, -1).Result()
	if err != nil {
		if err == redis.Nil {
			return nil
		}
		log.Println(err)
	}
	return val
}

// RedisDelListItem redis移除链表元素
func RedisDelListItem(key string, val string) {
	err := bot.GoRedis.LRem(ctx, key, 0, val).Err()
	if err != nil {
		log.Println(err)
	}
}

// RedisAddSet redis集合添加元素
func RedisAddSet(key string, val string) {
	err := bot.GoRedis.SAdd(ctx, key, val).Err()
	if err != nil {
		log.Println(err)
	}
}

// RedisSetIsExists redis集合是否包含元素
func RedisSetIsExists(key string, val string) bool {
	exists, err := bot.GoRedis.SIsMember(ctx, key, val).Result()
	if err != nil {
		log.Println(err)
	}
	return exists
}

// RedisDelSetItem redis移除集合元素
func RedisDelSetItem(key string, val string) {
	err := bot.GoRedis.SRem(ctx, key, val).Err()
	if err != nil {
		log.Println(err)
	}
}

// Screenshot 屏幕截图
func Screenshot(url string, waitTime float64, scale float64) []byte {
	pw, err := playwright.Run()
	if err != nil {
		log.Println("未检测到playwright，开始自动安装...")
		playwright.Install()
		pw, _ = playwright.Run()
	}
	browser, _ := pw.Chromium.Launch()
	page, _ := browser.NewPage(playwright.BrowserNewContextOptions{DeviceScaleFactor: &scale})
	defer func() {
		log.Println("关闭playwright")
		page.Close()
		browser.Close()
		pw.Stop()
	}()
	log.Println("开始进行截图...")
	page.Goto(url, playwright.PageGotoOptions{
		WaitUntil: playwright.WaitUntilStateNetworkidle,
	})
	page.WaitForTimeout(waitTime)
	locator, _ := page.Locator(".main")
	if v, _ := locator.IsVisible(); !v {
		log.Println("元素未加载取消截图操作")
		return nil
	}
	screenshot, err := locator.Screenshot(playwright.LocatorScreenshotOptions{Type: playwright.ScreenshotTypeJpeg})
	if err != nil {
		return nil
	}
	log.Println("截图完成...")
	return screenshot
}

// ReverseSlice 反转切片
func ReverseSlice[T any](s []T) {
	for i := 0; i < len(s)/2; i++ {
		j := len(s) - i - 1
		s[i], s[j] = s[j], s[i]
	}
}

// EscapesMarkdownV2 ModeMarkdownV2特殊字符转义
func EscapesMarkdownV2(s string) string {
	var i int
	for i = 0; i < len(s); i++ {
		if special(s[i]) {
			break
		}
	}
	if i >= len(s) {
		return s
	}

	b := make([]byte, 2*len(s)-i)
	copy(b, s[:i])
	j := i
	for ; i < len(s); i++ {
		if special(s[i]) {
			b[j] = '\\'
			j++
		}
		b[j] = s[i]
		j++
	}
	return string(b[:j])
}

func special(b byte) bool {
	var specialBytes [16]byte
	for _, b := range []byte(`_*[]()~>#+-=|{}.!`) {
		specialBytes[b%16] |= 1 << (b / 16)
	}
	specialBytes[byte('`')%16] |= 1 << (byte('`') / 16)
	return b < utf8.RuneSelf && specialBytes[b%16]&(1<<(b/16)) != 0
}

func Md5(str string) string {
	m5 := md5.Sum([]byte(str))
	m5str := hex.EncodeToString(m5[:])
	return m5str
}

func ImgConvert(url string) []byte {
	pic, err := http.Get(url)
	if err != nil {
		log.Println("获取图片失败")
		return nil
	}
	m, err := webp.Decode(pic.Body)
	if err != nil {
		log.Println("解析图片失败")
		return nil
	}
	bounds := m.Bounds()
	dx := bounds.Dx()
	dy := bounds.Dy()
	newRgba := image.NewRGBA(bounds)
	f := true
	go overtime(&f)
o:
	for i := 0; i < dx; i++ {
		for j := 0; j < dy; j++ {
			if !f {
				log.Println("图片转换超时")
				break o
			}
			colorRgb := m.At(i, j)
			r, g, b, a := colorRgb.RGBA()
			r_uint8 := uint8(r >> 8)
			g_uint8 := uint8(g >> 8)
			b_uint8 := uint8(b >> 8)
			a_uint8 := uint8(a >> 8)

			r_uint8 = g_uint8
			b_uint8 = g_uint8
			a_uint8 = 255
			if r_uint8 != 0 || g_uint8 != 0 || b_uint8 != 0 {
				r_uint8 = 255
				g_uint8 = 255
				b_uint8 = 255
			}
			newRgba.SetRGBA(i, j, color.RGBA{R: r_uint8, G: g_uint8, B: b_uint8, A: a_uint8})
		}
	}
	if !f {
		return nil
	}
	buf := new(bytes.Buffer)
	png.Encode(buf, newRgba)
	return buf.Bytes()
}

func CutImg(url string) []byte {
	pic, err := http.Get(url)
	if err != nil {
		log.Println("获取图片失败", err)
		return nil
	}
	m, err := webp.Decode(pic.Body)
	if err != nil {
		log.Println("解析图片失败", err)
		return nil
	}
	rgba := m.(*image.NYCbCrA)
	subImage := rgba.SubImage(image.Rect(0, m.Bounds().Dy(), m.Bounds().Dx(), int(float64(m.Bounds().Dy())/1.5))).(*image.NYCbCrA)
	buf := new(bytes.Buffer)
	png.Encode(buf, subImage)
	return buf.Bytes()
}

func overtime(f *bool) {
	time.Sleep(time.Second * 10)
	*f = false
}

func OCR(file io.Reader, lang, engine, sep string) []string {
	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)
	part, err := writer.CreateFormFile("file", "test.png")
	if err != nil {
		log.Println("创建文件失败")
		return nil
	}
	io.Copy(part, file)
	writer.WriteField("language", lang)
	writer.WriteField("FileType", ".Auto")
	writer.WriteField("OCREngine", engine)
	writer.Close()
	request, err := http.NewRequest("POST", "https://api.ocr.space/parse/image", body)
	if err != nil {
		log.Println(err)
		return nil
	}
	request.Header.Set("Content-Type", writer.FormDataContentType())
	request.Header.Add("Apikey", "helloworld")
	resp, err := http.DefaultClient.Do(request)
	if err != nil {
		log.Println("ocr失败")
		return nil
	}
	read, _ := io.ReadAll(resp.Body)
	defer resp.Body.Close()
	result := gjson.ParseBytes(read)
	log.Println("识别结果：", result.String())
	return strings.Split(result.Get("ParsedResults.0.ParsedText").String(), sep)
}
