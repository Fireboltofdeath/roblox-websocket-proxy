local HttpService = game:GetService("HttpService")

type WebSocketOptions = {
	url: string,
	batchMs: number?,
}

local CONNECT_URL = "/connect"
local GET_URL = "/%s/get?long=%s&batch_ms=%s"
local SEND_URL = "/%s/send"
local CLOSE_URL = "/%s/close"

local POLL_FAILURE_BACKOFF = 5

local WebSocket = {}
WebSocket.__index = WebSocket

WebSocket.ENDPOINT = "http://localhost:3000"

function WebSocket.new(options: WebSocketOptions)
	local self = setmetatable({}, WebSocket)

	self.connected = false
	self.url = options.url
	self.batchMs = options.batchMs
	self.onmessageCallbacks = {}
	self.oncloseCallbacks = {}

	return self
end

function WebSocket:_request(url, method, body)
	local response = HttpService:RequestAsync({
		Url = WebSocket.ENDPOINT .. url,
		Method = method,
		Body = body and HttpService:JSONEncode(body) or nil,
		Headers = {
			["Content-Type"] = "application/json",
		},
	})

	local responseBody = HttpService:JSONDecode(response.Body)
	if responseBody.success then
		return true, responseBody.result
	else
		return false, responseBody.error
	end
end

function WebSocket:_close(reason)
	self.connected = false

	for _, callback in self.oncloseCallbacks do
		task.spawn(callback, reason)
	end
end

function WebSocket:_incoming(message)
	if message.type == "content" then
		for _, callback in self.onmessageCallbacks do
			task.spawn(callback, message.content)
		end
	elseif message.type == "close" then
		self:_close(message.reason)
	end
end

function WebSocket:_startPolling()
	task.spawn(function()
		local pollUrl = string.format(GET_URL, self.socketId, "true", self.batchMs or "0")

		-- TODO: implement budget checks
		while self.connected do
			local success, response = self:_request(pollUrl, "GET")
			if not success then
				-- The socket is dead and/or got cleaned up.
				if response == "Socket not alive" or response == "Socket not found" then
					self:_close()
					break
				end

				warn("Poll failure:", response)
				task.wait(POLL_FAILURE_BACKOFF)
				continue
			end

			for _, message in response do
				self:_incoming(message)
			end
		end
	end)
end

function WebSocket:connect()
	if self.connected then
		error("Attempted to connect more than once.")
	end

	local success, connection = self:_request(CONNECT_URL, "POST", { url = self.url })
	if not success then
		return false
	end

	self.connected = true
	self.socketId = connection.socket_id
	self:_startPolling()

	return true
end

function WebSocket:send(data)
	if not self.connected then
		warn("Cannot send data to disconnected websocket")
		return
	end

	local sendUrl = string.format(SEND_URL, self.socketId)
	local success, message = self:_request(sendUrl, "POST", { data = data })
	if not success then
		warn("Error sending message:", message)
	end

	return success
end

function WebSocket:close()
	if not self.connected then
		warn("Cannot close disconnected websocket")
		return
	end

	local closeUrl = string.format(CLOSE_URL, self.socketId)
	local success, message = self:_request(closeUrl, "DELETE")
	if not success then
		warn("Error closing socket:", message)
	end

	return success
end

function WebSocket:onMessaged(callback)
	table.insert(self.onmessageCallbacks, callback)
end

function WebSocket:onClosed(callback)
	table.insert(self.oncloseCallbacks, callback)
end

return WebSocket
